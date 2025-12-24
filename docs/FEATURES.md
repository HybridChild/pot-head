# Features

## Dual Type Parameters

`PotHead<TIn, TOut = TIn>` supports separate input and output types:

```rust
// ADC → normalized float (common pattern)
let config: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    // ...
};
let mut pot = PotHead::new(config)?;
let volume: f32 = pot.update(adc_value);

// Same type (default)
let config: Config<u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 1000,
    // ...
};
let mut pot = PotHead::new(config)?;
let pwm: u16 = pot.update(adc_value);
```

*Default `TOut = TIn` allows concise type annotations for same-type cases.*

## Processing Pipeline

Input processing follows a fixed order:

```
Input (TIn)
  → Normalize to f32 (0.0-1.0)
  → Noise Filter
  → Response Curve
  → Hysteresis
  → Snap Zones
  → Grab Mode
  → Denormalize to TOut
  → Output (TOut)
```

## Response Curves

Transform normalized input through different response characteristics.

### Linear

1:1 mapping—output directly proportional to input:

```rust
curve: ResponseCurve::Linear,
```

*Always available.*

### Logarithmic

Audio taper for perceptually linear volume control:

```rust
curve: ResponseCurve::Logarithmic,
```

*Requires `std-math` feature. Uses exponential function for characteristic audio response.*

## Noise Filtering

Smooth noisy ADC readings. All filtering happens in normalized `f32` space.

### Exponential Moving Average (EMA)

Weighted average with previous output:

```rust
filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
```

- `alpha`: Smoothing factor (0.0 < α ≤ 1.0)
- Lower values = more smoothing, less responsive
- Higher values = less smoothing, more responsive

*RAM cost: 4 bytes per pot. Always available.*

### Moving Average

Simple average over N samples:

```rust
filter: NoiseFilter::MovingAverage { window_size: 8 },
```

- `window_size`: Number of samples to average (1-32)
- Predictable lag, consistent smoothing

*RAM cost: `window_size` × 4 bytes per pot. Requires `moving-average` feature.*

### No Filter

Disable filtering:

```rust
filter: NoiseFilter::None,
```

## Hysteresis

Prevent rapid output oscillation from noisy or boundary-crossing inputs.

### Change Threshold

Ignore changes smaller than threshold:

```rust
hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.05 },
```

Output only updates when input differs from last output by more than threshold. Effective for gradual jitter and noise.

*Operates on normalized values (0.0-1.0).*

### Schmitt Trigger

Separate rising and falling thresholds prevent boundary oscillation:

```rust
hysteresis: HysteresisMode::SchmittTrigger {
    rising: 0.6,
    falling: 0.4,
},
```

- When input ≥ `rising`: output = `rising`
- When input ≤ `falling`: output = `falling`
- Between thresholds: output maintains previous state

*Requires `rising > falling`. Ideal for digital-like behavior and preventing chatter at switching points.*

### No Hysteresis

Disable hysteresis:

```rust
hysteresis: HysteresisMode::none(),
```

## Snap Zones

Define regions where input behavior changes.

### Snap Zones

Lock output to target value when within threshold:

```rust
snap_zones: &[
    SnapZone::new(0.0, 0.05, SnapZoneType::Snap),  // Snap to 0% (±5%)
    SnapZone::new(0.5, 0.1, SnapZoneType::Snap),  // Snap to 50% (±10%)
    SnapZone::new(1.0, 0.05, SnapZoneType::Snap),  // Snap to 100% (±5%)
],
```

When normalized input falls within `target ± threshold`, output snaps to `target`.

### Dead Zones

Ignore input changes within zone:

```rust
snap_zones: &[
    SnapZone::new(0.5, 0.05, SnapZoneType::Dead),  // Dead zone at 50% (±5%)
],
```

Output holds previous value when input is within the dead zone range.

### Zone Processing

Multiple zones are processed in array order—first match wins. This allows intentional overlap for layered behavior:

```rust
snap_zones: &[
    SnapZone::new(0.0, 0.05, SnapZoneType::Dead),  // Dead zone ±5%
    SnapZone::new(0.0, 0.10, SnapZoneType::Snap),  // Snap zone ±10%
],
```

*Dead zone checked first, so within ±5% movement is ignored, but 5-10% snaps to 0.*

## Grab Modes

Prevent parameter jumps when physical pot position doesn't match virtual value (after preset changes or automation).

### Pickup Mode

Virtual value doesn't update until pot crosses it from below:

```rust
grab_mode: GrabMode::Pickup,
```

```
Scenario: Virtual = 70%, Physical = 20%
User moves pot: 20% → 50% → no output change
Pot crosses 70% → grabbed!
Further movement controls parameter normally
```

*Industry standard in professional audio equipment.*

### PassThrough Mode

Virtual value doesn't update until pot crosses it from either direction:

```rust
grab_mode: GrabMode::PassThrough,
```

```
Scenario A: Virtual = 70%, Physical = 20% (below)
User moves pot upward → catches at 70% ✓

Scenario B: Virtual = 30%, Physical = 80% (above)
User moves pot downward → catches at 30% ✓
```

*More intuitive UX—faster to grab, better for bidirectional controls.*

### No Grab Mode

Disable grab mode:

```rust
grab_mode: GrabMode::None,
```

*Physical pot immediately controls output (may cause jumps).*

### UI Support

Query physical position during grab mode for dual-state display:

```rust
let output = pot.update(raw_adc);

if pot.is_waiting_for_grab() {
    let physical = pot.physical_position();  // Where pot actually is
    let virtual_val = pot.current_output();  // Locked virtual value

    // Display both values to guide user
    display.show_bar(virtual_val, Color::Yellow);
    display.show_ghost_bar(physical, Color::Gray);
}
```

*Requires `grab-mode` feature. Adds ~24-40 bytes RAM per pot depending on output type.*

## Static ROM Configuration

v0.1 uses static configuration stored in flash memory (ROM), minimizing RAM usage:

```rust
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Logarithmic,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.05 },
    snap_zones: &[SnapZone::new(0.0, 0.02, SnapZoneType::Snap)],
    grab_mode: GrabMode::Pickup,
};

// Validate at compile time
const _: () = {
    match VOLUME_CONFIG.validate() {
        Ok(()) => {},
        Err(e) => panic!("{}", e),
    }
};

// Create instance (only state in RAM)
let mut pot = PotHead::new(VOLUME_CONFIG)?;
```

Multiple `PotHead` instances can share the same configuration, storing only runtime state in RAM.

## Compile-Time Validation

Configuration errors caught at compile time via const validation:

```rust
// This will fail to compile with clear error message
static BAD_CONFIG: Config<u16, f32> = Config {
    input_min: 100,
    input_max: 0,  // Error: input_min >= input_max
    // ...
};

const _: () = {
    match BAD_CONFIG.validate() {
        Ok(()) => {},
        Err(e) => panic!("{}", e),  // Compile error
    }
};
```

Validation checks:
- Input range: `input_min < input_max`
- Output range: `output_min ≠ output_max`
- Hysteresis: `rising > falling` (Schmitt trigger)
- Filter: Alpha in range (0.0, 1.0], window_size 1-32

*Optional `validate_snap_zones()` checks for overlaps if needed.*

## Runtime Behavior

Invalid inputs handled gracefully:

```rust
let output = pot.update(raw_adc);
```

- Out-of-range inputs: Clamped to `[input_min, input_max]`
- Numeric overflow: Wrapped in release, panics in debug
- ADC glitches: Absorbed by clamping and filtering

*No panics in release builds — embedded-friendly error handling.*

## Feature Flags

Enable only the functionality you need:

```toml
[dependencies]
pot-head = { version = "0.1", default-features = false, features = ["std-math"] }
```

### Available Features

| Feature | Default | Dependency | Enables |
|---------|---------|------------|---------|
| `std-math` | ✅ Yes | `libm` | Logarithmic response curves |
| `moving-average` | ❌ No | `heapless` | Moving average filter |
| `grab-mode` | ✅ Yes | None | Pickup/PassThrough grab modes |

### Default Configuration

```toml
default = ["std-math", "grab-mode"]
```

### Minimal Configuration

For maximum ROM/RAM efficiency:

```toml
pot-head = { version = "0.1", default-features = false }
```

Provides: Linear curves, EMA filter, change threshold hysteresis, snap zones.

## Complete Example

```rust
use pot_head::{
    Config, GrabMode, HysteresisMode, NoiseFilter, PotHead,
    ResponseCurve, SnapZone, SnapZoneType,
};

// Static configuration in ROM
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Logarithmic,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.05 },
    snap_zones: &[SnapZone::new(0.0, 0.02, SnapZoneType::Snap)],
    grab_mode: GrabMode::Pickup,
};

// Compile-time validation
const _: () = {
    match VOLUME_CONFIG.validate() {
        Ok(()) => {},
        Err(e) => panic!("{}", e),
    }
};

fn main() {
    // Create pot (only state in RAM)
    let mut volume_pot = PotHead::new(VOLUME_CONFIG).unwrap();

    loop {
        // Read hardware
        let raw_adc = read_adc_channel(0);

        // Process through pot-head
        let volume: f32 = volume_pot.update(raw_adc);

        // Apply to your application
        audio_driver.set_volume(volume);

        // Optional: UI display for grab mode
        if volume_pot.is_waiting_for_grab() {
            display_dual_state(
                volume_pot.current_output(),
                volume_pot.physical_position(),
            );
        }
    }
}
```

## Performance Characteristics

- **Zero allocations**: All processing uses stack or static storage
- **Predictable timing**: `update()` is deterministic, suitable for real-time
- **Minimal branching**: Linear processing pipeline optimizes for CPU cache
- **Feature compilation**: Disabled features don't exist in binary (zero overhead)

## Future Roadmap

Deferred to Future Versions:

1. **Builder API** - Fluent builder pattern for runtime configuration
2. **Calibration API** - Runtime helpers for learning physical pot ranges and center positions
3. **Advanced Grab Modes** - Scaling, threshold catch, takeover, timeout release
4. **Detent Simulation** - Magnetic resistance around configured points
5. **Advanced Noise Filters** - Median, Kalman-like, adaptive filtering
6. **Response Curve Extensions** - Exponential, S-curve, custom lookup tables
7. **Multi-Pot Features** - Ganging, master/slave relationships, crossfading
