# sizeof-calculator

Measures RAM usage for different **pot-head** configurations.

## Usage

```bash
cd tools/sizeof-calculator

# Generate report to stdout
cargo run --release

# Save report to file
cargo run --release > ../../reports/sizeof_report.md
```

## What It Measures

- **Component sizes**: Config, State, and PotHead for common type combinations
- **Filter impact**: RAM overhead of different filters
- **Feature flag impact**: Size differences between minimal/default/full features
- **Common configurations**: Real-world usage scenarios

## Output

Generates a markdown report showing:
- Actual type sizes (not estimates)
- RAM overhead per filter type
- Comparison across type combinations (u16→f32, u16→u16, etc.)
- Memory efficiency notes for embedded deployment

## Notes

- The tool measures sizes as built with the features specified in its Cargo.toml
- To see different feature combinations, modify the `pot-head` dependency features
- State<f32> size is constant regardless of TIn/TOut (pot-head normalizes internally)
- Config struct lives in Flash (ROM), only State consumes RAM at runtime
