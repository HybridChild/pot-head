# Filtering Example

Demonstrates noise filtering with pot-head.

## Features

This example shows how to use both filter types:

1. **EMA Filter** (Exponential Moving Average) - Responsive and smooth
2. **Moving Average Filter** - Predictable lag, good spike rejection
3. **No Filter** - Raw passthrough for comparison
4. **Combined Filtering + Hysteresis** - Professional noise handling

## Running

```bash
cargo run
```

## What You'll Learn

- How to configure filters in the `Config` struct
- The difference between EMA and Moving Average filtering
- How filters combine with hysteresis for robust input handling
- Practical filtering parameters for typical ADC noise

## Key Concepts

**EMA (Exponential Moving Average):**
- Single parameter: `alpha` (0.0 to 1.0)
- Higher alpha = more responsive, less smoothing
- Lower alpha = more smoothing, less responsive
- RAM: ~8 bytes per filter

**Moving Average:**
- Single parameter: `window_size` (1 to 32)
- Averages last N samples
- RAM: window_size × 4 bytes per filter

**NoiseFilter::None:**
- Zero overhead passthrough
- Use when filtering not needed
