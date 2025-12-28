# binary-analyzer

Flash footprint analysis tool for **pot-head** across ARM Cortex-M targets.

## What It Does

Measures binary size impact of feature flags to answer:
- **"How much Flash does each feature cost?"**
- **"Should I use an MCU with an FPU?"**
- **"Which features can I afford on my target?"**

Analyzes:
- `.text` section size (code in Flash)
- Total Flash footprint
- Symbol breakdown (what's using space)
- Feature cost comparison across targets

## Targets Analyzed

- **Cortex-M0+** (`thumbv6m-none-eabi`) - No FPU, soft-float
- **Cortex-M4F** (`thumbv7em-none-eabihf`) - Hardware FPU, hard-float

## Feature Combinations Tested

1. **minimal** - `--no-default-features` (linear curves only)
2. **default** - `std-math` + `grab-mode` (logarithmic curves)
3. **full** - `--all-features` (all features enabled)

## Requirements

**Required:**
- Rust toolchain
- `cargo-bloat` tool
- ARM embedded toolchain with `arm-none-eabi-size`
  - macOS: `brew install --cask gcc-arm-embedded`
  - Linux: `sudo apt-get install gcc-arm-none-eabi`

## Usage

```bash
# From the binary-analyzer directory
./generate_report.sh

# View the generated report
cat ../../reports/binary_report.md
```

The script will:
1. Install required Rust targets if needed
2. Build test binary for all target/feature combinations
3. Extract size information
4. Analyze top symbols
5. Generate markdown report with recommendations

## Output

Generates `reports/binary_report.md` with:

- **Flash Footprint Summary** - Size table for all combinations
- **Feature Cost Breakdown** - Cost per feature, FPU savings
- **Top Symbols by Size** - What's consuming space
- **Deployment Recommendations** - Guidance for different constraints

## Test Binary

The `test-binary/` directory contains a minimal `no_std` application that:
- Uses realistic static ROM configuration (pot-head v0.1 pattern)
- Exercises all code paths to prevent dead code elimination
- Compiles with aggressive size optimization (`opt-level = "z"`, LTO)
- Measures actual embedded binary size (not host builds)
