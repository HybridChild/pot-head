# pot-head: Design Specification

## Design Status

All major design decisions for v0.1 have been resolved. The specification is complete and ready for implementation.

**Key Decisions:**
1. **Numeric types**: Generic via `num-traits` with separate input/output types
2. **Error handling**: Build-time validation + runtime clamping + debug assertions
3. **API design**: Hybrid builder pattern and static ROM configs
4. **Configuration storage**: Both owned (RAM) and borrowed (ROM) via `ConfigRef`
5. **Feature gating**: Granular Cargo features for compile-time optimization

**Features included in v0.1:**
- Linear and logarithmic response curves
- EMA and moving average noise filters
- Schmitt trigger and change threshold hysteresis
- Snap and dead zones
- Pickup and PassThrough grab modes

**Features deferred:** See "Features Deferred to Future Versions" section below.

---

## Non-Goals

This crate explicitly does **NOT**:
- Perform any I/O operations (ADC reads, GPIO)
- Integrate with HAL crates (user's responsibility)
- Provide async/await support (synchronous function calls only)
- Handle interrupts (user calls `update()` from wherever)
- Allocate memory dynamically (everything stack-based)
- Provide hardware-specific optimizations

---

## Design Decisions & Alternatives

### 1. Numeric Type Support

#### Current Decision: Generic over numeric types via `num-traits` with separate input and output types

**Approach:** Use two type parameters (`TIn` for input, `TOut` for output) with trait bounds to support any numeric type combinations (u8, u16, u32, f32, f64, etc.). When both types are the same, `TOut` defaults to `TIn` for ergonomic API.

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Integer-only** (u8, u16, u32) | Simpler implementation<br>No FPU required<br>Guaranteed no_std | Limited to discrete values<br>Less flexible for output ranges<br>Harder to implement logarithmic curves |
| **Fixed-point arithmetic** | Fractional values without FPU<br>Deterministic performance | Additional dependency (`fixed` crate)<br>Less familiar API for users<br>Conversion overhead |
| **Single generic type** | Simple API<br>Supports both int and float | Forces same type for input and output<br>Wasteful conversions (ADC u16 → f32 → u16 PWM)<br>Prevents natural ADC→float normalization |
| **Separate input/output types (chosen)** | Natural ADC→application mapping<br>Type-safe boundaries<br>Eliminates wasteful conversions<br>Builder infers types from ranges | Slightly more complex implementation<br>Two type parameters to manage |

**Rationale:** The vast majority of embedded potentiometer applications follow the pattern: integer ADC input → normalized float processing → application-specific output. Supporting separate input/output types eliminates wasteful conversions that users would otherwise have to do manually, while the builder pattern's type inference makes this ergonomic.

**Type Parameter Design:**

```rust
pub struct PotHead<TIn, TOut = TIn>
where
    TIn: Numeric,
    TOut: Numeric,
{
    config: ConfigRef<TIn, TOut>,
    state: State<TOut>,  // State tracks output type
}
```

The default `TOut = TIn` means simple same-type cases remain concise (`PotHead<u16>` instead of `PotHead<u16, u16>`).

**Processing Flow:**

```
Input (TIn) → Normalize (f32) → Filter → Curve → Snap Zones → Denormalize (TOut) → Output (TOut)
                ↑                                                       ↑
              Convert to internal representation            Convert to output type
```

**Examples:**

```rust
// Example 1: Same input/output type (simple case)
// Type inference from builder - no explicit type parameters needed!
let mut pot = PotHead::builder()
    .input_range(0_u16..4095_u16)    // TIn inferred as u16
    .output_range(0_u16..1000_u16)   // TOut inferred as u16
    .build();
// Returns PotHead<u16, u16> (or just PotHead<u16> via default)

// Example 2: ADC → normalized float (most common embedded pattern)
let mut pot = PotHead::builder()
    .input_range(0_u16..4095_u16)     // TIn = u16 (12-bit ADC)
    .output_range(0.0_f32..1.0_f32)   // TOut = f32 (normalized)
    .response_curve(ResponseCurve::Logarithmic)
    .build();
// Returns PotHead<u16, f32>
// No wasteful conversions - ADC stays u16 until normalization

// Example 3: Float ADC → integer PWM
let mut pot = PotHead::builder()
    .input_range(0.0_f32..3.3_f32)    // TIn = f32 (voltage from floating ADC)
    .output_range(0_u16..65535_u16)   // TOut = u16 (16-bit PWM)
    .build();
// Returns PotHead<f32, u16>

// Example 4: Explicit type parameters (when inference insufficient)
let mut pot: PotHead<u16, f32> = PotHead::builder()
    .input_range(0..4095)
    .output_range(0.0..1.0)
    .build();

// Example 5: Static config with different types
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Logarithmic,
    // ... other config
};

let mut volume_pot = PotHead::from_static(&VOLUME_CONFIG);
```

**Implementation Notes:**

1. **Internal Processing Type:** All intermediate calculations (normalization, filtering, curves, snap zones) happen in a common internal type (typically `f32` or `f64` depending on FPU availability). This is automatically chosen based on the wider of `TIn` and `TOut`.

2. **Trait Bounds:** Both `TIn` and `TOut` must implement `num_traits::Numeric` (or similar trait providing arithmetic operations and conversions).

3. **Zero Overhead:** Type conversions only happen at boundaries (input normalization and output denormalization). Processing pipeline operates in single type.

4. **Builder Inference:** The builder pattern automatically infers `TIn` and `TOut` from the types of range bounds, making the API ergonomic without explicit type annotations in most cases.

**Why This Matters for Embedded:**

- **ADC → Float**: Very common pattern - 12-bit ADC (`u16`) to normalized 0.0-1.0 (`f32`) for audio/control parameters
- **ADC → Signed**: Pots controlling bipolar parameters (pan: `u16` → `i8` for -100 to +100)
- **Float → PWM**: Normalized calculations to hardware PWM duty cycle (`f32` → `u16`)
- **Memory Efficiency**: No wasteful storage of converted values - input stays in native type until processing

---

### 2. Hysteresis Implementation

#### Current Decision: Support both Schmitt trigger and change threshold modes

**Approach:** Provide an enum allowing users to choose the hysteresis mode that fits their needs

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Schmitt trigger only** | Classic approach<br>Well-understood behavior<br>Prevents boundary oscillation | Doesn't handle gradual noise/drift<br>Requires two thresholds |
| **Change threshold only** | Simple to configure<br>Single parameter<br>Effective for most noise | Doesn't prevent rapid oscillation at boundaries |
| **Both modes (chosen)** | Covers all use cases<br>Users pick what fits | Slightly larger API surface<br>Users must understand difference |

**Rationale:** Different noise patterns require different solutions. Schmitt triggers excel at preventing boundary oscillation (e.g., digital outputs), while change thresholds handle gradual jitter better. Supporting both adds minimal complexity.

**Example:**
```rust
// Schmitt trigger for digital-like behavior
.hysteresis(HysteresisMode::SchmittTrigger {
    rising: 2050,   // Switch high at this value
    falling: 2045,  // Switch low at this value
})

// Change threshold for analog smoothing
.hysteresis(HysteresisMode::ChangeThreshold(5))  // Ignore changes < 5 units
```

---

### 3. Snap Zones & Detent Simulation

#### Current Decision: Snap-to-value and dead zones; detent simulation deferred to v0.2+

**Approach:** Implement two snap zone types (snap and dead) with deferred detent simulation

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **All three types** (snap, dead, detent) | Complete feature set<br>Professional feel | Complex implementation<br>Unclear if detents are widely needed<br>May bloat binary |
| **Snap zones only** | Simplest implementation<br>Covers primary use case | Missing dead zone functionality<br>Less versatile |
| **Snap + Dead (chosen)** | Covers known requirements<br>Can add detents later if needed | Feature incomplete if detents turn out essential |

**Detent Simulation Explanation:**
- **Snap zones**: Binary - you're either snapped to target or not
- **Dead zones**: Input changes ignored within zone boundaries
- **Detent simulation**: Creates "magnetic resistance" around points - harder to move through certain values (like physical rotary encoder detents)

**Rationale:** Start with well-understood snap and dead zones. Detent simulation is more complex and less commonly needed. Can be added in future version based on user feedback.

**Example:**
```rust
// Snap to 0% when within 2%
.snap_zone(SnapZone {
    target: 0.0,
    threshold: 0.02,
    zone_type: SnapZoneType::Snap,
})

// Dead zone at 50% center position (±3%)
.snap_zone(SnapZone {
    target: 0.5,
    threshold: 0.03,
    zone_type: SnapZoneType::Dead,
})
```

---

#### Overlapping Snap Zones

**Current Decision: Priority by order (first match wins)**

**Problem:** When multiple snap zones are configured, their ranges (target ± threshold) might overlap. We need a deterministic way to handle this.

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Reject overlaps at build time** | Catches configuration errors early<br>Clear, unambiguous behavior<br>No runtime overhead | Too restrictive<br>Prevents intentional layering<br>Complex const validation |
| **Priority by zone type** (Dead > Snap) | Intuitive for some cases<br>Allows useful patterns | Doesn't generalize well<br>Complex when mixing types<br>May not match user intent |
| **Closest target wins** | Intuitive "magnetic" behavior<br>Allows overlaps | More computation per update<br>Complex edge cases (equidistant)<br>Harder to predict |
| **Priority by order (chosen)** | Simple implementation<br>Predictable behavior<br>Allows intentional overlaps<br>Zero runtime overhead | Requires understanding array order<br>Possible subtle bugs if unintentional |

**Rationale:** Priority by order is the simplest approach that provides maximum flexibility. Users who want overlapping zones (e.g., a dead zone that masks an underlying snap zone) can achieve this intentionally. Users who want to ensure no overlaps can use the optional validation helper.

**Implementation Details:**

```rust
impl SnapZone<T> {
    /// Check if this zone overlaps with another.
    /// Two zones overlap if their ranges (target ± threshold) intersect.
    pub fn overlaps(&self, other: &SnapZone<T>) -> bool {
        let self_min = self.target - self.threshold;
        let self_max = self.target + self.threshold;
        let other_min = other.target - other.threshold;
        let other_max = other.target + other.threshold;

        !(self_max < other_min || other_max < self_min)
    }
}

impl<T> Config<T> {
    /// Validate that no snap zones overlap.
    /// This is an optional validation helper - overlaps are allowed by default.
    /// Call this during development if you want to ensure clean, non-overlapping zones.
    pub fn validate_snap_zones(&self) -> Result<(), ConfigError> {
        for (i, zone1) in self.snap_zones.iter().enumerate() {
            for zone2 in self.snap_zones.iter().skip(i + 1) {
                if zone1.overlaps(zone2) {
                    return Err(ConfigError::OverlappingSnapZones {
                        zone1: *zone1,
                        zone2: *zone2,
                    });
                }
            }
        }
        Ok(())
    }
}

impl<T> PotHead<T> {
    fn apply_snap_zones(&self, value: T) -> T {
        // Process zones in order - first match wins
        for zone in self.config.snap_zones {
            if zone.contains(value) {
                return zone.apply(value);
            }
        }
        value  // No zone matched
    }
}
```

**Usage Example:**

```rust
// Example 1: Intentional overlap - dead zone masks snap zone
.snap_zone(SnapZone::new(0.0, 0.05, SnapZoneType::Snap))   // Snap to 0% (±5%)
.snap_zone(SnapZone::new(0.0, 0.02, SnapZoneType::Dead))   // Dead zone at 0% (±2%)
// Dead zone is checked first, so within ±2% movement is ignored,
// but from 2-5% it snaps to 0%

// Example 2: Non-overlapping zones
let config = Config {
    snap_zones: &[
        SnapZone::new(0.0, 0.02, SnapZoneType::Snap),
        SnapZone::new(0.5, 0.03, SnapZoneType::Dead),
        SnapZone::new(1.0, 0.02, SnapZoneType::Snap),
    ],
    // ... other config
};

// Optional: Validate no overlaps during development
#[cfg(debug_assertions)]
config.validate_snap_zones().expect("Snap zones must not overlap");
```

**Documentation Requirements:**

1. **API docs must clearly state**: "Multiple snap zones are processed in array order. If an input value falls within multiple zones, the first matching zone in the array takes priority."

2. **Examples should demonstrate**: Both intentional overlaps (layering behavior) and the validation helper for users who want to prevent overlaps.

3. **Performance note**: First-match-wins has optimal performance - O(n) where n is number of zones, with early exit on first match.

---

### 4. Error Handling Strategy

#### Current Decision: Hybrid approach with build-time validation, runtime clamping, and debug assertions

**Approach:** Multi-layered error handling that balances safety, performance, and developer experience

**The Core Problem:**

The `update()` method is called in tight loops (1-10ms intervals in embedded systems). Invalid states could occur from:
- Configuration errors (input_min >= input_max, invalid ranges)
- Numeric overflow/underflow during calculations
- Division by zero (e.g., input_min == input_max)
- Out-of-range inputs (ADC glitches, noise spikes)

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Always panic on invalid input** | Clear failure mode<br>Forces correct usage<br>Easy to debug | Unacceptable in embedded/real-time systems<br>ADC glitches cause crashes<br>No graceful degradation |
| **Always return Result** | Explicit error handling<br>Caller decides behavior | Runtime overhead every call<br>Verbose call sites<br>Unergonomic for hot path |
| **Silent clamping only** | Zero overhead<br>Graceful degradation<br>Handles ADC glitches | Masks configuration bugs<br>Hard to debug<br>Silent failures |
| **Debug assertions only** | Zero release overhead<br>Catches bugs in development | No safety net in production<br>Release builds can still crash |
| **Hybrid (chosen)** | Catches errors early (build time)<br>Graceful runtime behavior<br>Debug aids during development<br>Zero overhead in release | Most complex implementation<br>Multiple code paths to maintain |

**Rationale:** Embedded systems require different error handling than typical Rust applications. We can't panic on bad input (ADC glitches happen), but we also can't afford runtime overhead on every `update()` call. The solution: validate once at build time, clamp at runtime, assert in debug mode.

---

#### Implementation Strategy

**Layer 1: Build-Time Validation (Primary Defense)**

Catch configuration errors when the `PotHead` is constructed:

```rust
impl<T> PotHeadBuilder<T> {
    pub fn build(self) -> Result<PotHead<T>, ConfigError> {
        // Validate ranges (prevents division by zero)
        if self.input_min >= self.input_max {
            return Err(ConfigError::InvalidInputRange {
                min: self.input_min,
                max: self.input_max,
            });
        }

        if self.output_min >= self.output_max {
            return Err(ConfigError::InvalidOutputRange {
                min: self.output_min,
                max: self.output_max,
            });
        }

        // Validate hysteresis configuration
        #[cfg(feature = "hysteresis-schmitt")]
        if let HysteresisMode::SchmittTrigger { rising, falling } = self.hysteresis {
            if rising <= falling {
                return Err(ConfigError::InvalidSchmittThresholds { rising, falling });
            }
        }

        // Validate snap zones don't overlap (optional)
        if self.validate_snap_zones {
            self.validate_snap_zones()?;
        }

        Ok(PotHead {
            config: ConfigRef::Owned(self.into_config()),
            state: State::default(),
        })
    }
}

pub enum ConfigError {
    InvalidInputRange { min: T, max: T },
    InvalidOutputRange { min: T, max: T },
    InvalidSchmittThresholds { rising: T, falling: T },
    OverlappingSnapZones { zone1: SnapZone<T>, zone2: SnapZone<T> },
    InvalidFilterWindow { window_size: usize },
}
```

**Layer 2: Runtime Input Clamping (Graceful Degradation)**

Handle out-of-range inputs gracefully (ADC glitches, noise spikes):

```rust
impl<T> PotHead<T> {
    pub fn update(&mut self, input: T) -> T {
        // Clamp input to valid range
        // This handles ADC glitches and noise spikes gracefully
        let input = input.clamp(self.config.input_min, self.config.input_max);

        // All internal calculations are now safe because:
        // 1. Config validated at build time (no division by zero)
        // 2. Input is within valid range (no out-of-bounds)
        let normalized = self.normalize(input);
        let filtered = self.apply_filter(normalized);
        let snapped = self.apply_snap_zones(filtered);
        let curved = self.apply_curve(snapped);

        self.denormalize(curved)
    }

    fn normalize(&self, input: T) -> T {
        let range = self.config.input_max - self.config.input_min;
        // Safe: validated at build time that input_max > input_min
        // range is guaranteed non-zero
        (input - self.config.input_min) / range
    }
}
```

**Layer 3: Debug Assertions (Development Safety Net)**

Catch unexpected conditions during development without release overhead:

```rust
impl<T> PotHead<T> {
    pub fn update(&mut self, input: T) -> T {
        // In debug builds, catch unexpected out-of-range inputs
        // This helps identify configuration errors during development
        debug_assert!(
            input >= self.config.input_min && input <= self.config.input_max,
            "Input {} out of range [{}, {}] - possible configuration error or ADC malfunction",
            input, self.config.input_min, self.config.input_max
        );

        let input = input.clamp(self.config.input_min, self.config.input_max);
        // ... rest of implementation
    }

    fn denormalize(&self, normalized: T) -> T {
        let range = self.config.output_max - self.config.output_min;

        // Checked arithmetic in debug, wrapping in release
        #[cfg(debug_assertions)]
        {
            self.config.output_min
                .checked_add(normalized.checked_mul(range)
                    .expect("Overflow in output calculation"))
                .expect("Overflow in output calculation")
        }

        #[cfg(not(debug_assertions))]
        {
            self.config.output_min + (normalized * range)
        }
    }
}
```

**Layer 4: Optional Strict Mode (Feature-Gated)**

For safety-critical applications that want explicit error handling:

```rust
#[cfg(feature = "strict-runtime-checks")]
impl<T> PotHead<T> {
    /// Strict update that returns an error instead of clamping invalid input.
    /// Only available with the `strict-runtime-checks` feature.
    /// Use this in safety-critical applications where invalid input must be handled explicitly.
    pub fn update_strict(&mut self, input: T) -> Result<T, RuntimeError> {
        if input < self.config.input_min || input > self.config.input_max {
            return Err(RuntimeError::InputOutOfRange {
                input,
                min: self.config.input_min,
                max: self.config.input_max,
            });
        }

        // Proceed with normal processing
        Ok(self.update_internal(input))
    }
}

#[cfg(feature = "strict-runtime-checks")]
pub enum RuntimeError {
    InputOutOfRange { input: T, min: T, max: T },
}
```

---

#### Const Validation for Static Configs

For ROM-based configurations, we provide const validation helpers:

```rust
impl<T> Config<T> {
    /// Const helper to validate configuration at compile time.
    /// Note: Requires const assertions (may need const_panic or const_trait_impl features)
    pub const fn validate(&self) -> Result<(), &'static str> {
        // Basic compile-time checks
        if self.input_min >= self.input_max {
            return Err("Invalid input range: min >= max");
        }
        if self.output_min >= self.output_max {
            return Err("Invalid output range: min >= max");
        }
        Ok(())
    }
}

// Usage with static configs:
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    // ... rest of config
};

// Validate at compile time (requires const evaluation)
const _: () = {
    match VOLUME_CONFIG.validate() {
        Ok(()) => {},
        Err(e) => panic!("{}", e),  // Compile error if config invalid
    }
};
```

---

#### Error Handling Summary

| Error Type | Detection | Handling | Performance Impact |
|------------|-----------|----------|-------------------|
| **Invalid ranges** | Build time | Return `Err(ConfigError)` from `build()` | Zero (fails before runtime) |
| **Overlapping snap zones** | Build time (optional) | Return `Err(ConfigError)` if validation enabled | Zero (one-time check) |
| **Out-of-range input** | Runtime | Clamp to valid range | Minimal (1 comparison) |
| **Numeric overflow** | Debug only | Panic in debug, wrap in release | Zero in release |
| **Division by zero** | Build time | Prevented by range validation | Zero (impossible after validation) |

**Key Principle:** Error handling moves left - catch errors as early as possible (build time > startup > debug > runtime), minimizing production overhead while maximizing safety.

---

### 5. Noise Filtering

#### Current Decision: Basic averaging filters (EMA and moving average)

**Approach:** Provide exponential moving average (EMA) and simple moving average

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **No filtering** | Simplest<br>Users can add external filtering | Most users need this<br>Duplicates code across projects<br>Misses core value proposition |
| **Advanced filters** (median, Kalman) | Superior noise rejection<br>Handles outliers better | More complex implementation<br>Larger binary size<br>Diminishing returns for typical noise |
| **Basic filtering (chosen)** | Covers 90% of use cases<br>Minimal code<br>Well-understood behavior | Not optimal for extreme noise environments<br>May need tuning |

**Rationale:** EMA and moving average are simple, effective, and cover the vast majority of potentiometer noise scenarios. Advanced filtering can be added later if needed, but would increase complexity significantly.

**Example:**
```rust
// Exponential moving average (responsive, smooth)
.noise_filter(NoiseFilter::ExponentialMovingAverage { alpha: 0.3 })
// Lower alpha = more smoothing, higher = more responsive

// Simple moving average (predictable lag)
.noise_filter(NoiseFilter::MovingAverage { window_size: 8 })
```

---

### 6. Grab Mode Functionality

#### Current Decision: Support Pickup and PassThrough modes in v0.1

**Approach:** Provide two grab modes - Pickup (unidirectional catch) and PassThrough (bidirectional catch)

**The Problem:**

When physical pot position doesn't match virtual parameter value (e.g., after automation or preset change):

```
Virtual parameter: 70%
Physical pot position: 20%
User moves pot → without grab mode, parameter jumps to 20% (jarring!)
```

**Alternatives Considered:**

| Approach | Responsiveness | Jumps | Complexity | v0.1? |
|----------|----------------|-------|------------|-------|
| **Jump immediately** | High | Large | Very Low | ❌ No - unprofessional |
| **Pickup** (unidirectional) | Medium | None | Low | ✅ Yes - industry standard |
| **PassThrough** (bidirectional) | High | None | Low | ✅ Yes - UX improvement |
| **Scaling mode** | High | None | Medium | ❌ Defer - variable sensitivity confusing |
| **Threshold catch** | High | Small | Low | ❌ Defer - needs per-app tuning |
| **Takeover mode** (DAW-style) | Very High | None | High | ❌ Defer - complex state machine |
| **Timeout release** | Medium | None | Medium | ❌ Defer - requires timer/clock |

**Rationale:**
- **Pickup mode** is the industry standard in professional audio equipment and DAW controllers
- **PassThrough mode** is a natural extension with minimal complexity that significantly improves UX
- Both modes prevent parameter jumps and are intuitive for users
- More complex modes (scaling, takeover) deferred to v0.2 after gathering user feedback

---

#### Grab Mode Behaviors

**Pickup Mode (Unidirectional Catch):**

Virtual value doesn't update until physical pot crosses it from below:

```rust
.grab_mode(GrabMode::Pickup)

// Scenario: Virtual = 70%, Physical = 20%
// User moves pot: 20% → 30% → 50% → no output change
// Pot crosses 70% → grabbed!
// Further movement controls parameter normally
```

**PassThrough Mode (Bidirectional Catch):**

Virtual value doesn't update until physical pot crosses it from *either direction*:

```rust
.grab_mode(GrabMode::PassThrough)

// Scenario A: Virtual = 70%, Physical = 20% (pot below)
// User moves pot upward: catches at 70% ✓

// Scenario B: Virtual = 30%, Physical = 80% (pot above)
// User moves pot downward: catches at 30% ✓ (NEW!)
```

**Advantages of PassThrough:**
- Faster to grab (don't need to overshoot and come back)
- More intuitive (catches from whichever direction you approach)
- Better UX for bidirectional controls (pan, balance)
- Minimal complexity increase (~10 lines of code)

**Implementation:**

```rust
pub enum GrabMode {
    /// Disabled - pot position immediately controls output (may cause jumps)
    None,

    /// Pickup mode - catches when pot crosses virtual value from below
    Pickup,

    /// PassThrough mode - catches when pot crosses virtual value from either direction
    PassThrough,
}

impl<T> PotHead<T> {
    fn update(&mut self, input: T) -> T {
        match self.config.grab_mode {
            GrabMode::None => {
                // Direct control, no grab logic
                self.process_input(input)
            }

            GrabMode::Pickup => {
                if !self.state.grabbed {
                    if input >= self.state.virtual_value {
                        self.state.grabbed = true;
                    } else {
                        return self.state.virtual_value; // Hold virtual value
                    }
                }
                self.process_input(input)
            }

            GrabMode::PassThrough => {
                if !self.state.grabbed {
                    let crossing_from_below =
                        input >= self.state.virtual_value &&
                        self.state.last_physical < self.state.virtual_value;

                    let crossing_from_above =
                        input <= self.state.virtual_value &&
                        self.state.last_physical > self.state.virtual_value;

                    if crossing_from_below || crossing_from_above {
                        self.state.grabbed = true;
                    } else {
                        self.state.last_physical = input;
                        return self.state.virtual_value; // Hold virtual value
                    }
                }
                self.state.last_physical = input;
                self.process_input(input)
            }
        }
    }
}
```

**State Requirements:**

```rust
pub struct State<TOut> {
    pub grabbed: bool,
    pub virtual_value: TOut,

    #[cfg(feature = "grab-mode")]
    pub physical_position: TOut,  // Processed input position (for UI queries)

    #[cfg(feature = "grab-mode")]
    pub last_physical: TOut,  // Only needed for PassThrough mode (crossing detection)

    // ... other state fields
}
```

---

#### UI Support for Grab Mode

**Problem:** When grab mode is active but not yet grabbed, the output is locked at the virtual value while the pot is physically at a different position. Professional UX requires showing both values to guide the user.

**Solution:** Store the physical position in state and provide methods to query both the virtual (locked) output and the current physical position:

```rust
impl<TIn, TOut> PotHead<TIn, TOut> {
    /// Returns the current output value (may be locked if grab mode active)
    pub fn update(&mut self, input: TIn) -> TOut {
        // Process through normalize→filter→curve
        let normalized = self.normalize(input);
        let filtered = self.apply_filter(normalized);
        let curved = self.apply_curve(filtered);

        // Capture physical position BEFORE snap zones and grab mode
        // This represents where the pot physically is (with filtering and curve applied)
        // but before virtual modifications like snap zones
        self.state.physical_position = curved;

        // Apply virtual modifications (snap zones)
        let snapped = self.apply_snap_zones(curved);

        // Apply grab mode logic
        let output = match self.config.grab_mode {
            GrabMode::None => snapped,
            GrabMode::Pickup | GrabMode::PassThrough => {
                if self.state.grabbed {
                    snapped
                } else {
                    // Check grab conditions, update grabbed state...
                    self.state.virtual_value  // Return locked value if not grabbed
                }
            }
        };

        self.state.virtual_value = output;
        output
    }

    /// Returns the current physical input position in output units.
    /// Useful for UI display when grab mode is active.
    ///
    /// This always reflects where the pot physically is (after normalize→filter→curve),
    /// but BEFORE virtual modifications like snap zones and grab mode logic.
    ///
    /// Compare this with `current_output()` to show grab mode status in UI.
    /// The difference shows what virtual modifications are currently applied.
    ///
    /// This is a simple field access with zero overhead - the physical position
    /// is computed once per `update()` call and stored in state.
    ///
    /// # Example
    /// ```rust
    /// let output = pot.update(raw_adc);
    ///
    /// if pot.is_waiting_for_grab() {
    ///     let physical = pot.physical_position();  // Where pot actually is
    ///     let virtual_val = pot.current_output();  // Locked/snapped value
    ///     display.show_both(output, physical);
    /// }
    /// ```
    #[cfg(feature = "grab-mode")]
    pub fn physical_position(&self) -> TOut {
        self.state.physical_position
    }

    /// Returns the current output value without updating state.
    /// Useful for reading the locked virtual value in grab mode.
    pub fn current_output(&self) -> TOut {
        self.state.virtual_value
    }

    /// Returns true if grab mode is active but not yet grabbed.
    /// When true, `physical_position() != current_output()`
    #[cfg(feature = "grab-mode")]
    pub fn is_waiting_for_grab(&self) -> bool {
        matches!(self.config.grab_mode, GrabMode::Pickup | GrabMode::PassThrough)
            && !self.state.grabbed
    }
}
```

**Usage Example:**

```rust
// Professional UI showing grab mode status
let output = volume_pot.update(raw_adc);

if volume_pot.is_waiting_for_grab() {
    // Pot not yet grabbed - show dual state
    let physical = volume_pot.physical_position();
    let virtual_val = volume_pot.current_output();

    // Display both values
    display.show_label("Volume (locked)");
    display.show_bar(virtual_val, Color::Yellow);      // Current parameter value
    display.show_ghost_bar(physical, Color::DimGray);  // Physical pot position

    // Show directional hint
    if physical < virtual_val {
        display.show_icon(Icon::ArrowUp);    // "Move pot up to grab"
    } else {
        display.show_icon(Icon::ArrowDown);  // "Move pot down to grab"
    }
} else {
    // Normal operation - pot is grabbed or grab mode disabled
    display.show_label("Volume");
    display.show_bar(output, Color::Green);
}
```

**State Storage Cost:**

- Adds `TOut` to state for `physical_position` (typically 4-8 bytes for f32/f64)
- Computed once per `update()` call (no reprocessing overhead)
- `physical_position()` method is O(1) field access
- Only when `grab-mode` feature enabled

**Why This Matters:**

Professional equipment (DAW controllers, digital mixers, synthesizers) universally shows this dual state during pickup mode. Without it, users get confused: "I'm moving the pot but nothing's happening!" With proper UI feedback, the behavior is intuitive and predictable.

---

### 7. API Design

#### Current Decision: Hybrid approach with both builder pattern and static configuration

**Approach:** Support both fluent builder API (for flexibility) and static ROM-based configuration (for resource-constrained systems)

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Configuration struct** | Explicit parameter passing<br>Configuration is serializable<br>Can be const | Less discoverable<br>Verbose initialization<br>No type safety on methods |
| **Const generic builder** | Zero-cost abstraction<br>Compile-time validation<br>No runtime storage | Inflexible after initialization<br>Complex compile errors<br>Can't adjust thresholds at runtime<br>Code duplication for each unique config |
| **Builder pattern only** | Fluent, discoverable API<br>Flexible configuration<br>Clear method names | Config stored in RAM<br>Not optimal for ROM-constrained systems |
| **Hybrid (chosen)** | Best of both worlds<br>User chooses based on constraints<br>Config can live in ROM or RAM | Slightly larger API surface |

**Rationale:** Different embedded systems have different constraints. The hybrid approach allows:
- **ROM-critical systems** (many pots, limited flash): Use static configs in ROM with shared code
- **RAM-critical systems** (few pots, limited RAM): Use builder pattern for minimal per-instance overhead
- **Flexible systems**: Use builder pattern for runtime reconfiguration

**ROM Storage Analysis:**
```
Single pot:        Const generic (~1KB ROM) < Hybrid (~3KB ROM)
5+ unique pots:    Hybrid (~3KB ROM) < Const generic (~5KB+ ROM)
20 pots, 3 types:  Hybrid (~3KB ROM) = Const generic (~3KB ROM)
                   BUT hybrid uses 80% less RAM
```

**Examples:**

```rust
// Approach 1: Builder pattern (config in RAM, max flexibility)
let mut pot = PotHead::builder()
    .input_range(0..4095)
    .output_range(0.0..1.0)
    .response_curve(ResponseCurve::Linear)
    .hysteresis(HysteresisMode::ChangeThreshold(5))
    .snap_zone(SnapZone::new(0.0, 0.02, SnapZoneType::Snap))
    .noise_filter(NoiseFilter::ExponentialMovingAverage { alpha: 0.3 })
    .enable_grab_mode(true)
    .build();

// Approach 2: Static ROM configuration (minimal RAM, shared code)
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Logarithmic,
    hysteresis: HysteresisMode::ChangeThreshold(8),
    snap_zones: &[SnapZone::new(0.0, 0.02, SnapZoneType::Snap)],
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    grab_mode: GrabMode::Pickup,
};

static PAN_CONFIG: Config<u16, i16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: -100,
    output_max: 100,
    curve: ResponseCurve::Linear,
    hysteresis: HysteresisMode::None,
    snap_zones: &[SnapZone::new(0, 5, SnapZoneType::Dead)],
    filter: NoiseFilter::None,
    grab_mode: GrabMode::None,
};

let mut volume_pot = PotHead::from_static(&VOLUME_CONFIG);
let mut pan_pot = PotHead::from_static(&PAN_CONFIG);
// Config lives in ROM (flash), only state lives in RAM
```

---

### 8. Response Curves

#### Current Decision: Linear and logarithmic (audio taper)

**Approach:** Support both linear and logarithmic response curves

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Linear only** | Simplest<br>Most common<br>No transcendental functions | Audio applications need log curves<br>Limited functionality |
| **Linear + Log (chosen)** | Covers both common use cases<br>Professional audio support | Requires floating-point or lookup tables<br>Complexity for log implementation |
| **Arbitrary curves** (user-defined) | Maximum flexibility<br>Custom curves possible | Much more complex API<br>Higher overhead<br>Rarely needed |

**Rationale:** Linear and logarithmic curves cover 95% of applications. Logarithmic is essential for audio (volume, frequency). Custom curves can be added later if there's demand.

**Example:**
```rust
// Linear for most controls
.response_curve(ResponseCurve::Linear)

// Logarithmic for volume/audio parameters
.response_curve(ResponseCurve::Logarithmic)
```

---

### 9. Feature Gating

#### Current Decision: Granular feature flags for compile-time optimization

**Approach:** Use Cargo feature flags to allow users to include only the functionality they need, reducing binary size and RAM usage.

**Rationale:** Embedded systems have vastly different resource constraints. A feature that's essential for professional audio equipment (grab mode, logarithmic curves) may be unnecessary bloat for a simple industrial control panel. Feature-gating allows:
- **Minimal systems**: Include only core functionality (linear response, basic hysteresis)
- **Audio applications**: Enable full feature set (log curves, filters, grab mode, snap zones)
- **Custom profiles**: Mix and match features based on specific requirements

Unlike complex runtime configuration, feature flags work at compile time with zero overhead—disabled features literally don't exist in the binary.

**Key Advantage:** No complex conditional logic in the signal processing flow. Each feature compiles out completely when disabled, maintaining code clarity and performance.

---

#### Feature Flag Design

**Core Features:**

| Feature | Default | ROM Impact | RAM Impact | Description |
|---------|---------|------------|------------|-------------|
| `builder` | ✅ Yes | ~2-3 KB | 0 bytes | Fluent builder API (alternative: use `from_static()` only) |
| `log-curve` | ❌ No | ~2-5 KB | 0 bytes | Logarithmic response curves (requires `libm`) |
| `filter-ema` | ✅ Yes | ~0.5 KB | 4-8 bytes/pot | Exponential moving average filter |
| `filter-moving-avg` | ❌ No | ~1 KB | 32-128 bytes/pot | Simple moving average filter (requires buffer) |
| `hysteresis-threshold` | ✅ Yes | ~0.3 KB | 4 bytes/pot | Change threshold hysteresis |
| `hysteresis-schmitt` | ❌ No | ~0.4 KB | 8 bytes/pot | Schmitt trigger hysteresis |
| `snap-zone-snap` | ❌ No | ~0.8 KB | 0 bytes | Snap-to-target zones |
| `snap-zone-dead` | ❌ No | ~0.6 KB | 0 bytes | Dead zones (ignore input in range) |
| `grab-mode` | ❌ No | ~0.5-1 KB | 8-16 bytes/pot | Pickup/catch mode for parameter automation |

**Convenience Bundles:**

| Bundle | Includes | Use Case |
|--------|----------|----------|
| `default` | `builder`, `hysteresis-threshold`, `filter-ema` | General-purpose development |
| `full` | All features | Professional audio equipment, maximum functionality |
| `audio` | `log-curve`, `filter-ema`, `grab-mode`, `snap-zone-snap` | Audio mixers, synthesizers, DAW controllers |
| `industrial` | `hysteresis-threshold`, `filter-moving-avg` | Industrial control panels, machine interfaces |
| `minimal` | None (empty default-features) | Maximum ROM/RAM efficiency |

---

## Features Deferred to Future Versions

This section documents features that were considered for v0.1 but deferred to future releases. These decisions keep v0.1 focused, lean, and production-ready while providing a clear roadmap for enhancements.

### v0.2+ Planned Features

1. **Calibration API** - Runtime helpers for learning physical pot ranges and center positions
2. **Advanced Grab Modes** - Scaling, threshold catch, takeover, timeout release
3. **Detent Simulation** - Magnetic resistance around configured points
4. **Advanced Noise Filters** - Median, Kalman-like, adaptive filtering
5. **Response Curve Extensions** - Exponential, S-curve, custom lookup tables
6. **Multi-Pot Features** - Ganging, master/slave relationships, crossfading
