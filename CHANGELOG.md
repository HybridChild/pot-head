# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-03-13

### Fixed
- `detach()` now preserves the snapped output value as the virtual value instead of the raw pre-snap physical position.

## [0.2.0] - 2026-03-10

### Added
- `PotHead::attach(current_input: TIn)` — attaches the physical pot to this parameter; seeds the EMA filter to the current physical position to prevent false grabs (requires `grab-mode` feature)
- `PotHead::detach()` — detaches the physical pot from this parameter; if the pot was grabbed, snaps virtual value to current physical position so re-attaching requires passing through it again; if not yet grabbed, leaves virtual value unchanged to preserve the stored parameter value (requires `grab-mode` feature)

### Fixed
- PassThrough mode could falsely trigger a grab when a `PotHead` instance was reactivated after being idle: the stale EMA filter would ramp toward the true physical position over several `process()` calls, which the crossing detector misread as physical pot movement

### Changed
- `PotHead::update()` renamed to `PotHead::process()` — better reflects its role as a signal processing pipeline
- `PotHead::set_virtual_value(value: f32)` now takes `TOut` instead of normalized `f32` — value is in the same output space as `process()` returns
- `EmaFilter::reset` now takes a seed value `reset(value: f32)` and initializes the filter to that value, rather than cold-resetting to uninitialized state

## [0.1.0] - 2025-12-28

### Added
- Core `PotHead<TIn, TOut>` implementation with dual type parameters
- Response curves: Linear and Logarithmic (requires `std-math` feature)
- Noise filters: Exponential Moving Average and Moving Average (requires `moving-average` feature)
- Hysteresis modes: Schmitt Trigger and Change Threshold
- Snap zones and dead zones for flexible control configuration
- Grab modes: Pickup and PassThrough (requires `grab-mode` feature)
- Static ROM configuration pattern with compile-time validation
- Feature flags: `std-math` (default), `moving-average`, `grab-mode` (default)
- Comprehensive test suite with unit and integration tests
- Interactive terminal example demonstrating all features
- Complete documentation in `docs/FEATURES.md`
- Performance benchmarks for RP2040 and RP2350
- Binary size analysis and reports
- CI workflows for testing, documentation, and binary analysis
- MIT and Apache-2.0 dual licensing

[0.2.1]: https://github.com/HybridChild/pot-head/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/HybridChild/pot-head/releases/tag/v0.2.0
[0.1.0]: https://github.com/HybridChild/pot-head/releases/tag/v0.1.0
