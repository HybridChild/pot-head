# Interactive Example

Terminal-based demonstration of **pot-head** features.

## Overview

Simulates multiple potentiometers with different configurations, allowing you to explore all library features interactively. Demonstrates response curves, filters, hysteresis, snap zones, and grab modes in real-time.

## Running

```bash
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| `←` / `→` | Move potentiometer left/right |
| `↑` / `↓` | Select different potentiometer |
| `+` / `-` | Increase/decrease noise level |
| `q` / `Esc` | Quit |

## Features Demonstrated

Each potentiometer showcases different pot-head configurations:

- **Response curves** - Linear and logarithmic
- **Noise filters** - EMA and moving average with adjustable noise
- **Hysteresis** - Change threshold and Schmitt trigger
- **Snap zones** - Snap-to values and dead zones
- **Grab modes** - Pickup and PassThrough behavior

## Display

- **Green bar** - Output value range
- **Cyan marker** - Processed output position
- **Orange marker** - Physical input position (during grab mode)
- **Blue zones** - Snap zones
- **Gray zones** - Dead zones
- **Value display** - Shows output value with configured precision

## Implementation

The example uses a terminal UI built with `crossterm` and simulates ADC noise with configurable intensity. Source code demonstrates:

- Static ROM configuration patterns
- Multiple pots with different type parameters (`u16→f32`, `u16→u16`)
- Trait-based abstraction for rendering
- Real-time parameter updates

See `src/pot_spec.rs` for configuration examples.
