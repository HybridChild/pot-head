# pot-head

A `no_std` Rust library for processing potentiometer inputs in embedded systems.

[![Platform](https://img.shields.io/badge/platform-no_std-blue)](https://github.com/HybridChild/pot-head)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](https://github.com/HybridChild/pot-head)

## Overview

**pot-head** transforms raw ADC values into clean, processed output values through configurable filters, curves, and response modes.

The library provides a complete processing pipeline for analog inputs in resource-constrained embedded systems. Perfect for audio equipment, industrial control panels, and any embedded device with physical controls.

**Design Philosophy:** Pure mathematical abstraction—no I/O, no interrupts, no HAL integration. Just transformations.

---

## Key Features

### Core Functionality

- ✅ **Dual type parameters** - Separate input (ADC) and output types (`PotHead<u16, f32>`)
- ✅ **Response curves** - Linear and logarithmic
- ✅ **Noise filtering** - Exponential moving average and moving average
- ✅ **Hysteresis modes** - Schmitt trigger and change threshold for stability
- ✅ **Snap zones** - Snap-to values and dead zones for flexible control configuration
- ✅ **Grab modes** - Pickup and PassThrough for avoiding output jumps
- ✅ **Static ROM config** - Zero-copy configuration in flash memory

### What This Library Excludes

- ❌ Hardware I/O (ADC reads, GPIO, timers)
- ❌ HAL integration
- ❌ Interrupt handling
- ❌ Dynamic memory allocation

See `docs/FEATURES.md` for complete feature documentation.

### Processing Pipeline

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

---

## Quick Start

### Add Dependency

```toml
[dependencies]
pot-head = "0.1"

# Optional features
pot-head = { version = "0.1", features = ["std-math", "moving-average", "grab-mode"] }
```

### Example - Logarithmic Volume Control

```rust
use pot_head::{PotHead, Config, ResponseCurve, NoiseFilter, HysteresisMode, SnapZone, SnapZoneType};

// Define static configuration (stored in flash, not RAM)
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,        // 12-bit ADC
    output_min: -60.0,      // -60dB to 0dB (silence to unity gain)
    output_max: 0.0,
    curve: ResponseCurve::Logarithmic,  // Requires 'std-math' feature
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::ChangeThreshold(8),
    snap_zones: &[
        SnapZone::new(-60.0, 0.02, SnapZoneType::Snap),  // Snap to min (silence)
        SnapZone::new(0.0, 0.02, SnapZoneType::Snap),    // Snap to max (unity gain)
    ],
    grab_mode: GrabMode::PassThrough,
};

// Validate at compile time
const _: () = {
    match VOLUME_CONFIG.validate() {
        Ok(()) => {},
        Err(e) => panic!("Invalid config"),
    }
};

// Create potentiometer instance (minimal RAM usage)
let mut volume_pot = PotHead::new(&VOLUME_CONFIG);

// In your main loop:
loop {
    let adc_value: u16 = read_adc(); // Your hardware-specific ADC read
    let volume_db: f32 = volume_pot.update(adc_value);

    set_audio_volume(volume_db); // Your application logic
}
```

---

## Documentation

- **[FEATURES.md](docs/FEATURES.md)** - Complete feature reference with usage examples
- **[Interactive Example](examples/interactive/README.md)** - Full working demonstration (terminal-based)
- **`cargo doc --open`** - API documentation

---

## Feature Flags

| Feature | Description | Dependencies | Default |
|---------|-------------|--------------|---------|
| `std-math` | Logarithmic response curves | `libm` | ✅ |
| `grab-mode` | Pickup/PassThrough modes | None | ✅ |
| `moving-average` | Moving average filter | `heapless` | ❌ |

---

## Memory Footprint

**Static ROM Configuration:**
- Config stored in flash: TBD (depends on snap zones)
- Runtime state in RAM: TBD per instance

**Typical costs:**
- Base instance (no grab-mode): TBD bytes RAM
- With grab-mode: TBD bytes RAM
- Filter state: Included in base cost
- Moving average buffer: `WINDOW_SIZE × sizeof(TIn)` bytes (if enabled)

**Example:** A mixer with 8 faders using `PotHead<u16, f32>` with EMA filter and grab-mode:
- **Flash:** TBD (configs + code)
- **RAM:** TBD

---

## Performance

**Update cycle** (`update()` call):
- Typical: TBD on Cortex-M0+ @ 125 MHz
- With logarithmic curve: TBD (requires `f32` operations)

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

## Acknowledgments

Designed for the Rust embedded ecosystem.

**Maintained by:** Esben Dueholm Nørgaard ([HybridChild](https://github.com/HybridChild))
