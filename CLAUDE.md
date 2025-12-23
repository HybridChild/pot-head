# pot-head Development Guide

## Project Status

**Current Phase:** Design → Implementation

This is a Rust `no_std` embedded library for processing potentiometer inputs. The design is complete and documented in `docs/design-spec.md`. We're now ready to implement the core functionality.

## What pot-head Does

Transforms raw ADC values into clean, processed output values:
```
Raw ADC Value → pot-head Processing → Clean Output Value
```

**Core Features (v0.1):**
- Linear and logarithmic response curves
- EMA and moving average noise filters
- Schmitt trigger and change threshold hysteresis
- Snap and dead zones
- Pickup and PassThrough grab modes

## Project Structure

```
pot-head/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── config.rs           # Configuration types (Config)
│   ├── state.rs            # Runtime state management
│   ├── pothead.rs          # Core PotHead implementation
│   ├── filters/            # Noise filtering implementations
│   ├── curves/             # Response curve implementations
│   ├── hysteresis/         # Hysteresis mode implementations
│   ├── snap_zones/         # Snap zone implementations
│   └── grab_mode/          # Grab mode implementations
├── docs/
│   └── design-spec.md      # Complete design specification
├── examples/               # Usage examples (to be created)
└── tests/                  # Integration tests (to be created)
```

## Key Design Principles

### 1. No I/O, Pure Math
This library does NO hardware interaction. It's a pure transformation function. Users handle ADC reads and apply the library's processing.

### 2. Dual Type Parameters
```rust
PotHead<TIn, TOut = TIn>
```
- `TIn`: Input type (typically ADC integer like `u16`)
- `TOut`: Output type (often normalized `f32` or application-specific)
- Default `TOut = TIn` for same-type cases

### 3. Static ROM Configuration (v0.1)
v0.1 uses static ROM configuration exclusively:
- **Static ROM config** (config in flash, minimal RAM)
- **Builder pattern** deferred to v0.2+ (alongside calibration features)

### 4. Feature-Gated Compilation
Three meaningful features control dependencies and optional functionality:
```toml
default = ["std-math"]
std-math = ["libm"]           # Logarithmic curves
moving-average = ["heapless"] # Moving average filter
grab-mode = []                # Pickup/PassThrough modes (~24-40 bytes)
```

### 5. Error Handling Strategy
- **Const validation**: Catch config errors at compile time
- **Runtime clamping**: Gracefully handle ADC glitches
- **Debug assertions**: Catch issues in development
- **No panics in release**: Embedded-friendly

## Implementation Guidelines

### Code Organization
- Each major feature in its own module
- Feature-gate only dependency-driven code (libm, heapless) and grab-mode
- Keep processing pipeline in `PotHead::update()` clean and linear

### Processing Pipeline
```rust
Input (TIn)
  → Normalize (f32)
  → Filter
  → Curve
  → Snap Zones
  → Grab Mode
  → Denormalize (TOut)
  → Output (TOut)
```

### Performance Requirements
- `update()` called in tight loops (1-10ms intervals)
- Zero allocations (stack only)
- Minimal branching in hot path
- All processing modes compiled in by default (minimal overhead)

### Testing Strategy
- Unit tests for each module
- Integration tests for full pipeline
- Property-based tests for numeric edge cases
- Example programs demonstrating common use cases

## Common Patterns

### Static ROM Config Example (v0.1)
```rust
static VOLUME_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Logarithmic,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::ChangeThreshold(8),
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
let mut pot = PotHead::new(&VOLUME_CONFIG);

// In main loop:
let volume: f32 = pot.update(adc_value);
```

## Documentation Requirements

- All public APIs need doc comments
- Feature-gated items should document the required feature (std-math, moving-average, grab-mode)
- Demonstrate common use cases in interactive example
- Safety notes for `unsafe` code (if any)
- Performance characteristics (especially for filters)

## Dependencies

```toml
[dependencies]
num-traits = { version = "0.2", default-features = false }
libm = { version = "0.2", optional = true }        # For logarithmic curves (std-math feature)
heapless = { version = "0.9", optional = true }    # For moving average filter (moving-average feature)
```

## Reference Documentation

**Full Design Specification:** `docs/design-spec.md`
- Complete rationale for all design decisions
- Alternative approaches considered
- Detailed API examples
- Deferred features and roadmap

**Key Sections:**
- Numeric type design (separate TIn/TOut)
- Error handling strategy (const validation + runtime clamping)
- Feature gating details
- Grab mode implementations
- Overlapping snap zones behavior
- Builder API (deferred to v0.2+)

## Development Workflow

1. **Read the design spec** - Understand the complete picture before implementing
2. **Start with core types** - `Config`, `State`, `PotHead` skeleton
3. **Implement features incrementally** - One feature module at a time
4. **Test as you go** - Unit tests for each module
5. **Examples validate API** - Use interactive example to demonstrate features
6. **Lean and clean Document** - Brief API docs. No verbose doc comments

## Questions During Implementation

If design decisions need clarification or revision:
1. Check `docs/design-spec.md` for rationale
2. Consider if the change affects other design decisions
3. Update both implementation and design docs together
4. Note any deviations from the spec and why

---

**Remember:** This library is for embedded systems. Prioritize:
- Zero allocations
- Predictable performance
- Minimal binary size
- Clear, maintainable code
