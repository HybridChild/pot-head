# Virtual-Pot: Design Specification

## Problem Statement

Embedded developers working with physical controls (potentiometers, faders, sliders) face common challenges:
- **Noisy ADC readings** causing jittery output
- **Parameter jumps** when physical position doesn't match virtual state
- **Lack of professional polish** (no snap zones, no smooth response curves)
- **Boilerplate code** reimplemented in every project

This crate provides a reusable, zero-allocation, `no_std` solution for processing potentiometer inputs with professional-grade features.

## Core Principle

**Virtual-Pot is a pure mathematical abstraction.** It transforms raw input values (typically ADC readings) into processed output values based on configuration and internal state. The crate handles no I/O, no interrupts, no HAL integration - just math.

```
Raw ADC Value → Virtual-Pot Processing → Clean Output Value
```

## Target Use Cases

- **Audio equipment**: Mixers, synthesizers, effects processors (parameter automation with fetch/grab mode)
- **Industrial control panels**: Machine interfaces requiring noise immunity and reliability
- **Consumer devices**: Any embedded system with physical controls for human interaction

---

## Design Status

All major design decisions for v0.1 have been resolved. The specification is complete and ready for implementation.

**Key Decisions:**
1. **Numeric types**: Generic via `num-traits` (supports int and float)
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

#### Current Decision: Generic over numeric types via `num-traits`

**Approach:** Use trait bounds to support any numeric type (u8, u16, u32, f32, f64, etc.)

**Alternatives Considered:**

| Approach | Pros | Cons |
|----------|------|------|
| **Integer-only** (u8, u16, u32) | Simpler implementation<br>No FPU required<br>Guaranteed no_std | Limited to discrete values<br>Less flexible for output ranges<br>Harder to implement logarithmic curves |
| **Fixed-point arithmetic** | Fractional values without FPU<br>Deterministic performance | Additional dependency (`fixed` crate)<br>Less familiar API for users<br>Conversion overhead |
| **Generic (chosen)** | Maximum flexibility<br>Supports both int and float<br>User chooses based on hardware | Slightly more complex implementation<br>Requires `num-traits` dependency |

**Rationale:** Flexibility is crucial for embedded applications. Some systems have FPU and benefit from float precision; others need pure integer math. Let the user decide based on their constraints.

**Example:**
```rust
// Integer-based for simple 8-bit ADC
let mut pot_u8: VirtualPot<u8> = VirtualPot::builder()
    .input_range(0..255)
    .output_range(0..100)
    .build();

// Float-based for normalized output
let mut pot_f32: VirtualPot<f32> = VirtualPot::builder()
    .input_range(0.0..4095.0)
    .output_range(0.0..1.0)
    .build();
```

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

impl<T> VirtualPot<T> {
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

Catch configuration errors when the `VirtualPot` is constructed:

```rust
impl<T> VirtualPotBuilder<T> {
    pub fn build(self) -> Result<VirtualPot<T>, ConfigError> {
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

        Ok(VirtualPot {
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
impl<T> VirtualPot<T> {
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
impl<T> VirtualPot<T> {
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
impl<T> VirtualPot<T> {
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
static VOLUME_CONFIG: Config<u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 1000,
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

impl<T> VirtualPot<T> {
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
pub struct State<T> {
    pub grabbed: bool,
    pub virtual_value: T,

    #[cfg(feature = "grab-mode")]
    pub last_physical: T,  // Only needed for PassThrough mode

    // ... other state fields
}
```

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
let mut pot = VirtualPot::builder()
    .input_range(0..4095)
    .output_range(0.0..1.0)
    .response_curve(ResponseCurve::Linear)
    .hysteresis(HysteresisMode::ChangeThreshold(5))
    .snap_zone(SnapZone::new(0.0, 0.02, SnapZoneType::Snap))
    .noise_filter(NoiseFilter::ExponentialMovingAverage { alpha: 0.3 })
    .enable_grab_mode(true)
    .build();

// Approach 2: Static ROM configuration (minimal RAM, shared code)
static VOLUME_CONFIG: Config<u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 1000,
    curve: ResponseCurve::Logarithmic,
    hysteresis: HysteresisMode::ChangeThreshold(8),
    snap_zones: &[SnapZone::new(0, 20, SnapZoneType::Snap)],
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    grab_mode: true,
};

static PAN_CONFIG: Config<u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: -100,
    output_max: 100,
    curve: ResponseCurve::Linear,
    hysteresis: HysteresisMode::None,
    snap_zones: &[SnapZone::new(0, 5, SnapZoneType::Dead)],
    filter: NoiseFilter::None,
    grab_mode: false,
};

let mut volume_pot = VirtualPot::from_static(&VOLUME_CONFIG);
let mut pan_pot = VirtualPot::from_static(&PAN_CONFIG);
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

#### Implementation Example

**Cargo.toml Feature Configuration:**

```toml
[features]
# Default features for general use
default = ["builder", "hysteresis-threshold", "filter-ema"]

# Core API
builder = []

# Response curves
log-curve = ["libm"]

# Noise filtering
filters = ["filter-ema", "filter-moving-avg"]
filter-ema = []
filter-moving-avg = []

# Hysteresis modes
hysteresis = ["hysteresis-schmitt", "hysteresis-threshold"]
hysteresis-schmitt = []
hysteresis-threshold = []

# Snap zones
snap-zones = ["snap-zone-snap", "snap-zone-dead"]
snap-zone-snap = []
snap-zone-dead = []

# Grab mode
grab-mode = []

# Convenience bundles
full = ["builder", "log-curve", "filters", "hysteresis", "snap-zones", "grab-mode"]
audio = ["builder", "log-curve", "filter-ema", "grab-mode", "snap-zone-snap", "hysteresis-threshold"]
industrial = ["builder", "hysteresis-threshold", "filter-moving-avg"]
minimal = []  # Explicit empty bundle

[dependencies]
num-traits = { version = "0.2", default-features = false }
libm = { version = "0.2", optional = true }
heapless = { version = "0.8", optional = true }  # Only for moving-avg buffer

[package.metadata.docs.rs]
all-features = true  # Documentation shows all features
```

---

#### Code Structure with Feature Gates

**Response Curves:**
```rust
pub enum ResponseCurve {
    Linear,
    #[cfg(feature = "log-curve")]
    Logarithmic,
}

impl<T> VirtualPot<T> {
    fn apply_curve(&self, normalized: T) -> T {
        match self.config.curve {
            ResponseCurve::Linear => normalized,
            #[cfg(feature = "log-curve")]
            ResponseCurve::Logarithmic => {
                libm::log10f(normalized * 9.0 + 1.0)
            }
        }
    }
}
```

**Noise Filters:**
```rust
pub enum NoiseFilter {
    None,
    #[cfg(feature = "filter-ema")]
    ExponentialMovingAverage { alpha: f32 },
    #[cfg(feature = "filter-moving-avg")]
    MovingAverage { window_size: usize },
}

pub struct State<T> {
    last_output: T,

    #[cfg(feature = "filter-ema")]
    ema_state: Option<T>,

    #[cfg(feature = "filter-moving-avg")]
    moving_avg_buffer: heapless::Vec<T, 16>,
}
```

**Grab Mode:**
```rust
pub struct Config<T> {
    pub input_min: T,
    pub output_min: T,
    // ... other fields

    #[cfg(feature = "grab-mode")]
    pub grab_mode: bool,
}

impl<T> VirtualPot<T> {
    pub fn update(&mut self, input: T) -> T {
        let processed = self.apply_filtering(input);

        #[cfg(feature = "grab-mode")]
        if self.config.grab_mode {
            return self.handle_grab_mode(processed);
        }

        processed
    }
}
```

---

#### Usage Examples by Profile

**Minimal Profile (Industrial Control):**
```toml
# Cargo.toml
[dependencies]
virtual-pot = { version = "0.1", default-features = false }
```

```rust
// Only linear response, no filtering, no special features
// ROM: ~1-2 KB, RAM: ~8 bytes/pot
static SPEED_CONFIG: Config<u16> = Config {
    input_min: 0,
    input_max: 1023,
    output_min: 0,
    output_max: 100,
    curve: ResponseCurve::Linear,
    hysteresis: HysteresisMode::None,
    // snap_zones, filter, grab_mode don't exist when features disabled
};

let mut speed_pot = VirtualPot::from_static(&SPEED_CONFIG);
```

**Default Profile (General Purpose):**
```toml
# Cargo.toml
[dependencies]
virtual-pot = "0.1"  # Uses default features
```

```rust
// Builder API, EMA filter, change threshold hysteresis
// ROM: ~3-4 KB, RAM: ~32 bytes/pot
let mut pot = VirtualPot::builder()
    .input_range(0..4095)
    .output_range(0..100)
    .hysteresis(HysteresisMode::ChangeThreshold(5))
    .noise_filter(NoiseFilter::ExponentialMovingAverage { alpha: 0.3 })
    .build();
```

**Audio Profile (Professional Audio):**
```toml
# Cargo.toml
[dependencies]
virtual-pot = { version = "0.1", features = ["audio"] }
```

```rust
// Logarithmic curves, grab mode, EMA, snap zones
// ROM: ~5-6 KB, RAM: ~48 bytes/pot
static VOLUME_CONFIG: Config<u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 1000,
    curve: ResponseCurve::Logarithmic,
    hysteresis: HysteresisMode::ChangeThreshold(8),
    snap_zones: &[SnapZone::new(0, 20, SnapZoneType::Snap)],
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    grab_mode: true,
};
```

**Custom Profile:**
```toml
# Cargo.toml - Pick exactly what you need
[dependencies]
virtual-pot = {
    version = "0.1",
    default-features = false,
    features = ["log-curve", "hysteresis-schmitt", "grab-mode"]
}
```

---

#### Real-World Impact Example

**Scenario:** 16-channel mixer on STM32F103 (64KB flash, 20KB RAM)

| Configuration | ROM Usage | RAM Usage | Total |
|---------------|-----------|-----------|-------|
| **Full features** (all enabled) | ~7 KB | ~2048 bytes (16×128) | **~9 KB** |
| **Audio profile** | ~5.5 KB | ~768 bytes (16×48) | **~6.3 KB** |
| **Default profile** | ~3.5 KB | ~512 bytes (16×32) | **~4 KB** |
| **Minimal profile** | ~1.5 KB | ~128 bytes (16×8) | **~1.6 KB** |

**Savings:** Up to **82% reduction** in memory footprint by selecting appropriate features.

---

## API Overview

### Core Types

```rust
pub struct VirtualPot<T> {
    config: ConfigRef<T>,  // Either owned or borrowed from static
    state: State<T>,       // Runtime state (filter history, grab state, etc.)
}

enum ConfigRef<T> {
    Owned(Config<T>),              // From builder (config in RAM)
    Static(&'static Config<T>),    // From from_static (config in ROM)
}

pub struct Config<T> {
    pub input_min: T,
    pub input_max: T,
    pub output_min: T,
    pub output_max: T,
    pub curve: ResponseCurve,
    pub hysteresis: HysteresisMode<T>,
    #[cfg(any(feature = "snap-zone-snap", feature = "snap-zone-dead"))]
    pub snap_zones: &'static [SnapZone<T>],
    #[cfg(any(feature = "filter-ema", feature = "filter-moving-avg"))]
    pub filter: NoiseFilter,
    #[cfg(feature = "grab-mode")]
    pub grab_mode: bool,
}

pub struct VirtualPotBuilder<T> {
    // Fluent configuration builder
}

pub enum ResponseCurve {
    Linear,
    #[cfg(feature = "log-curve")]
    Logarithmic,
}

pub enum HysteresisMode<T> {
    None,
    #[cfg(feature = "hysteresis-schmitt")]
    SchmittTrigger { rising: T, falling: T },
    #[cfg(feature = "hysteresis-threshold")]
    ChangeThreshold(T),
}

#[cfg(any(feature = "snap-zone-snap", feature = "snap-zone-dead"))]
pub struct SnapZone<T> {
    pub target: T,
    pub threshold: T,
    pub zone_type: SnapZoneType,
}

#[cfg(any(feature = "snap-zone-snap", feature = "snap-zone-dead"))]
pub enum SnapZoneType {
    #[cfg(feature = "snap-zone-snap")]
    Snap,  // Hard snap to target value
    #[cfg(feature = "snap-zone-dead")]
    Dead,  // Ignore movement within zone
}

#[cfg(any(feature = "filter-ema", feature = "filter-moving-avg"))]
pub enum NoiseFilter {
    None,
    #[cfg(feature = "filter-ema")]
    ExponentialMovingAverage { alpha: f32 },
    #[cfg(feature = "filter-moving-avg")]
    MovingAverage { window_size: usize },
}
```

### Usage Example

```rust
#![no_std]
#![no_main]

use virtual_pot::{VirtualPot, Config, ResponseCurve, HysteresisMode, SnapZone, SnapZoneType, NoiseFilter};

// Example 1: Static ROM configuration (best for multiple pots, minimal RAM)
static VOLUME_CONFIG: Config<u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 1000,
    curve: ResponseCurve::Logarithmic,
    hysteresis: HysteresisMode::ChangeThreshold(8),
    snap_zones: &[SnapZone { target: 0, threshold: 20, zone_type: SnapZoneType::Snap }],
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    grab_mode: true,
};

#[entry]
fn main() -> ! {
    let mut adc = setup_adc();

    // Using static config (config in ROM, only state in RAM)
    let mut volume_fader = VirtualPot::from_static(&VOLUME_CONFIG);

    loop {
        let raw = adc.read_channel(0);
        let volume = volume_fader.update(raw);
        audio_codec.set_volume(volume);
        delay_ms(10);
    }
}
```

```rust
// Example 2: Builder pattern (best for runtime flexibility)
#[entry]
fn main() -> ! {
    let mut adc = setup_adc();

    // Using builder (config in RAM, can be modified at runtime)
    let mut volume_fader = VirtualPot::builder()
        .input_range(0..4095)           // 12-bit ADC
        .output_range(0..1000)          // 0-1000 range
        .response_curve(ResponseCurve::Logarithmic)
        .hysteresis(HysteresisMode::ChangeThreshold(8))
        .snap_zone(SnapZone::new(0, 20, SnapZoneType::Snap))
        .noise_filter(NoiseFilter::ExponentialMovingAverage { alpha: 0.3 })
        .enable_grab_mode(true)
        .build();

    loop {
        let raw = adc.read_channel(0);
        let volume = volume_fader.update(raw);
        audio_codec.set_volume(volume);
        delay_ms(10);
    }
}
```

---

## Technical Constraints

### No-STD Compatible
- Zero heap allocation
- No standard library dependencies
- Suitable for bare-metal embedded systems
- Stack-based state management

### Dependencies
```toml
[dependencies]
num-traits = { version = "0.2", default-features = false }
libm = { version = "0.2", optional = true }        # For logarithmic curves
heapless = { version = "0.8", optional = true }    # For moving average buffer

[features]
default = ["builder", "hysteresis-threshold", "filter-ema"]

# Core API
builder = []

# Response curves
log-curve = ["libm"]

# Noise filtering
filters = ["filter-ema", "filter-moving-avg"]
filter-ema = []
filter-moving-avg = ["heapless"]

# Hysteresis modes
hysteresis = ["hysteresis-schmitt", "hysteresis-threshold"]
hysteresis-schmitt = []
hysteresis-threshold = []

# Snap zones
snap-zones = ["snap-zone-snap", "snap-zone-dead"]
snap-zone-snap = []
snap-zone-dead = []

# Grab mode
grab-mode = []

# Convenience bundles
full = ["builder", "log-curve", "filters", "hysteresis", "snap-zones", "grab-mode"]
audio = ["builder", "log-curve", "filter-ema", "grab-mode", "snap-zone-snap", "hysteresis-threshold"]
industrial = ["builder", "hysteresis-threshold", "filter-moving-avg"]
minimal = []
```

---

## Features Deferred to Future Versions

This section documents features that were considered for v0.1 but deferred to future releases. These decisions keep v0.1 focused, lean, and production-ready while providing a clear roadmap for enhancements.

---

### v0.2+ Planned Features

#### 1. Calibration API

**What it is:** Runtime helpers for learning physical pot ranges and center positions.

**The problem:**
- Physical pots rarely use full electrical range (e.g., spec: 0-4095, actual: 150-3890 due to mechanical stops)
- Center detents don't align with electrical center (pan/balance controls)
- Manufacturing tolerances in taper curves

**Why deferred:**
- Application-specific (pro audio needs it, simple controls don't)
- Platform-specific persistence (EEPROM, flash, user's responsibility)
- Requires user interaction (UI/UX out of scope for core library)
- Can be implemented externally by wrapping `VirtualPot`
- v0.1 focuses on core signal processing, not application features

**Potential API:**
```rust
pot.calibrate_min();              // Record physical minimum
pot.calibrate_max();              // Record physical maximum
pot.calibrate_center();           // Record center detent position
pot.enable_auto_range();          // Passive learning over time
pot.apply_calibration()?;         // Update config with learned values
```

**External implementation example:**
```rust
struct CalibratedPot<T> {
    pot: VirtualPot<T>,
    original_config: Config<T>,
}

impl<T> CalibratedPot<T> {
    pub fn calibrate_min(&mut self) { /* record current position */ }
    pub fn calibrate_max(&mut self) { /* record current position */ }
    pub fn apply_calibration(&mut self) -> Result<(), CalibrationError> {
        // Rebuild VirtualPot with new input_min/input_max
    }
}
```

---

#### 2. Advanced Grab Modes

**What it is:** Additional grab behaviors beyond Pickup and PassThrough.

**Why deferred:**
- Pickup and PassThrough cover 95% of use cases
- Other modes significantly more complex
- Need user feedback on which modes are actually needed

**Deferred modes:**

| Mode | Description | Use Case |
|------|-------------|----------|
| **Scaling** | Pot movement scaled relative to virtual position | Motorized faders, touch screens |
| **Threshold Catch** | Grab when within N% of virtual value | Forgiving UX, consumer devices |
| **Takeover** | Scaling until caught, then pickup | Pro DAW controllers |
| **Timeout Release** | Auto-release grab after inactivity | Multi-user, automated systems |

See section 6 (Grab Mode Functionality) for detailed analysis of all modes.

---

#### 3. Detent Simulation

**What it is:** Creates "magnetic resistance" around configured points, simulating physical detents.

**Why deferred:**
- More complex than snap/dead zones
- Unclear if widely needed
- Snap zones cover most requirements
- Can add based on user demand

**How it would work:**
```rust
.snap_zone(SnapZone {
    target: 0.5,
    threshold: 0.03,
    zone_type: SnapZoneType::Detent {
        resistance: 0.7,  // 0.0 = no resistance, 1.0 = full snap
    },
})

// Within ±3% of 50%:
// - Movement is slowed by 70%
// - Creates "magnetic" feel without hard snapping
// - Like a physical rotary encoder detent
```

**Use cases:**
- Center detents on pan/balance controls
- Volume controls with "unity gain" detent
- Simulating mechanical feedback in digital controls

---

#### 4. Advanced Noise Filters

**What it is:** More sophisticated filtering beyond EMA and moving average.

**Why deferred:**
- Basic filters cover 90% of potentiometer noise scenarios
- Advanced filters significantly increase complexity and binary size
- Diminishing returns for typical embedded applications
- Can be added if users encounter extreme noise environments

**Potential filters:**

| Filter | Benefit | Complexity | Binary Size Impact |
|--------|---------|------------|-------------------|
| **Median filter** | Excellent outlier rejection | Medium | +1-2 KB |
| **Kalman-like** | Optimal noise reduction | High | +3-5 KB |
| **Adaptive filter** | Auto-tunes to noise conditions | Very High | +5-8 KB |

**Example future API:**
```rust
#[cfg(feature = "filter-median")]
pub enum NoiseFilter {
    // ... existing filters
    Median { window_size: usize },
}

#[cfg(feature = "filter-kalman")]
pub enum NoiseFilter {
    // ... existing filters
    Kalman { process_noise: f32, measurement_noise: f32 },
}
```

---

#### 5. Response Curve Extensions

**What it is:** Additional response curves beyond linear and logarithmic.

**Why deferred:**
- Linear and log curves cover 95% of applications
- Custom curves add significant complexity
- Can be added based on user feedback
- Users can implement custom curves externally if needed

**Potential curves:**

```rust
pub enum ResponseCurve {
    Linear,
    Logarithmic,

    #[cfg(feature = "curve-exponential")]
    Exponential,

    #[cfg(feature = "curve-s-curve")]
    SCurve { steepness: f32 },  // Ease-in/ease-out

    #[cfg(feature = "curve-custom")]
    Custom {
        lookup_table: &'static [T],  // Pre-computed curve
    },
}
```

**Use cases:**
- Exponential: Frequency controls (opposite of log)
- S-Curve: Smooth acceleration/deceleration
- Custom: Hardware-specific compensation curves

---

#### 6. Multi-Pot Features

**What it is:** Linking multiple pots together with relationships.

**Why deferred:**
- Niche requirement (most apps use independent pots)
- Significantly increases API complexity
- State management across multiple pots
- Better handled at application level

**Potential features:**

**Ganging:**
```rust
// Link multiple pots - moving one moves all
let gang = PotGang::new(&mut [pot1, pot2, pot3]);
gang.update(raw_input);  // Updates all pots in sync
```

**Master/Slave with Offset:**
```rust
// Slave pot follows master with offset/scaling
pot_slave.follow(&pot_master, |master_val| {
    master_val * 0.8 + 0.1  // Slave is 80% of master, +10% offset
});
```

**Crossfading:**
```rust
// Two pots create complementary outputs (sum to 100%)
let (out_a, out_b) = crossfade(pot_a, pot_b);
```

---

#### 7. Additional Enhancements

**Acceleration Curves:**
- Different response based on movement speed
- Fast movement = different scaling than slow movement
- Use case: DJ-style controls, quick parameter sweeps

**Deadband at Endpoints:**
- Prevent overrun at 0% and 100%
- Makes it easier to hit exact min/max
- Alternative to snap zones at endpoints

**Value Change Callbacks:**
- Notify when output value changes
- Useful for triggering side effects
- Requires closure/function pointer support

**Preset Storage:**
- Save/restore virtual positions
- Scene recall for multiple pot states
- Requires serialization support

---

### Version Roadmap (Tentative)

| Version | Focus | Key Features |
|---------|-------|--------------|
| **v0.1** | Core signal processing | Linear/log curves, EMA/moving-avg filters, hysteresis, snap/dead zones, pickup/passthrough grab modes |
| **v0.2** | Calibration & UX | Calibration API, threshold catch grab mode, detent simulation |
| **v0.3** | Advanced filtering | Median/Kalman filters, advanced grab modes (takeover, scaling) |
| **v1.0** | Production ready | Stabilized API, comprehensive testing, advanced response curves |

**Note:** Roadmap subject to change based on user feedback and real-world usage patterns.

---
