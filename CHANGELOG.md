# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `PotHead::reset_filter(current_input: TIn)` — seeds the EMA filter to the current physical position before `set_virtual_value()`, preventing false PassThrough grabs on reactivation (requires `grab-mode` feature)

### Fixed
- PassThrough mode could falsely trigger a grab when a `PotHead` instance was reactivated after being idle: the stale EMA filter would ramp toward the true physical position over several `update()` calls, which the crossing detector misread as physical pot movement

### Changed
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

[Unreleased]: https://github.com/HybridChild/pot-head/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/HybridChild/pot-head/releases/tag/v0.1.0
