# pot-head

A `no_std` Rust library for processing raw potmeter inputs in embedded systems.

[![Platform](https://img.shields.io/badge/platform-no_std-blue)](https://github.com/HybridChild/pot-head)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](https://github.com/HybridChild/pot-head)

## Overview

**pot-head** transforms raw ADC values into clean, processed output values through configurable noise filtering, response shaping, and grab modes.

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

// Define static configuration (stored in flash)
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,        // 12-bit ADC
    output_min: -60.0,      // -60dB to 0dB (silence to unity gain)
    output_max: 0.0,
    curve: ResponseCurve::Logarithmic,  // Requires 'std-math' feature
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.01 },  // 1% threshold
    snap_zones: &[
        SnapZone::new(-60.0, 0.02, SnapZoneType::Snap),  // Snap to min below 2%
        SnapZone::new(0.0, 0.02, SnapZoneType::Snap),    // Snap to max above 98%
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

// Create potentiometer instance from static config
let mut volume_pot = PotHead::new(&VOLUME_CONFIG);

// In your main loop:
loop {
    let adc_value: u16 = read_adc(); // Your hardware-specific ADC read
    let volume_db: f32 = volume_pot.process(adc_value);

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
- Config stored in flash: 40-44 bytes (depends on type parameters)
- Runtime state in RAM: 188 bytes per instance

**Typical costs:**
- `PotHead<u16, u16>` (integer scaling): 188 bytes RAM
- `PotHead<u16, f32>` (typical audio/control): 188 bytes RAM
- Filter state: Included in base cost
- Moving average buffer: `WINDOW_SIZE × 4` bytes additional (if enabled)

**Binary sizes (library logic):**
- **Cortex-M0+ (no FPU):** 1.7KB (minimal) to 2.9KB (default with std-math)
- **Cortex-M4F (with FPU):** 198B (minimal) to 624B (full features)

*Measured on ARM Cortex-M4F/M7 (`thumbv7em-none-eabihf`). See [`reports/sizeof_report.md`](reports/sizeof_report.md) and [`reports/binary_report.md`](reports/binary_report.md) for detailed analysis.*

---

## Performance

**Update cycle** (`process()` call):

| Platform | Configuration | Time | Cycles |
|----------|--------------|------|--------|
| **RP2040** (Cortex-M0+, 125 MHz, no FPU) | Baseline (linear, no filter) | 8.99µs | 1,123 |
| | With EMA filter | 12.72µs | 1,590 |
| | With logarithmic curve | 32.52µs | 4,065 |
| | Full featured | 47.02µs | 5,877 |
| **RP2350** (Cortex-M33F, 150 MHz, with FPU) | Baseline (linear, no filter) | 0.86µs | 128 |
| | With EMA filter | 1.00µs | 150 |
| | With logarithmic curve | 1.76µs | 264 |
| | Full featured | 2.28µs | 341 |

*See [`reports/rp2040_benchmarks.md`](reports/rp2040_benchmarks.md) and [`reports/rp2350_benchmarks.md`](reports/rp2350_benchmarks.md) for complete benchmark data.*

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
