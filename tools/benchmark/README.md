# pot-head Hardware Benchmarks

Hardware performance benchmarks for **pot-head** using real RP2040 and RP2350 microcontrollers.

## Overview

This tool measures the cycle-accurate performance of `PotHead::update()` on real ARM Cortex-M hardware:

- **RP2040** - Cortex-M0+ @ 125 MHz, soft-float (no FPU)
- **RP2350** - Cortex-M33F @ 150 MHz, hard-float (FPU)

## Hardware Requirements

You need at least one of:

- Raspberry Pi Pico (RP2040)
- Raspberry Pi Pico 2 (RP2350)

## Software Requirements

Install probe-rs for flashing and RTT output:

```bash
cargo install probe-rs-tools
```

## Running Benchmarks

### RP2040

```bash
cd tools/benchmark/rp2040
./run_benchmark.sh
```

Generates: `reports/rp2040_benchmarks.md`

### RP2350

```bash
cd tools/benchmark/rp2350
./run_benchmark.sh
```

Generates: `reports/rp2350_benchmarks.md`

## Benchmark Scenarios

The tool benchmarks 7 key scenarios:

1. **Baseline** - Linear curve, no filter (minimal processing)
2. **With EMA** - EMA filter (alpha=0.3)
3. **With Log Curve** - Logarithmic curve (shows FPU benefit)
4. **Full Featured** - EMA + log + hysteresis + snap zones + grab mode (worst case)
5. **MA Window=4** - MovingAverage filter (window=4)
6. **MA Window=16** - MovingAverage filter (window=16)
7. **u16→u16** - Integer-only path (u16→u16 vs u16→f32)

## Measurement Details

- **Warmup:** 100 iterations to stabilize CPU state
- **Measurement:** 1000 iterations for statistical accuracy
- **Optimization:** `-O3` with LTO enabled
- **Input:** u16 = 2048 (12-bit ADC mid-range value)
- **Duration:** 8 seconds per benchmark run

### Markdown Report

Each platform generates a markdown report in `reports/`:

- `reports/rp2040_benchmarks.md`
- `reports/rp2350_benchmarks.md`

Reports include:
- Timestamp and toolchain version
- Target architecture details
- Complete benchmark results

## How It Works

### Timer-Based Measurement

Both platforms use hardware timers for precise cycle counting:

- **RP2040:** 1 MHz timer at `0x40054000`
- **RP2350:** 1 MHz timer at `0x400B0000`

Microseconds are converted to CPU cycles at the appropriate frequency (125 MHz or 150 MHz).

### Measurement Protocol

```rust
// Warmup (100 iterations)
for _ in 0..100 { f(); asm::dmb(); }

// Measure (1000 iterations)
let start = read_timer();
for _ in 0..1000 { f(); asm::dmb(); }
let end = read_timer();
```

Memory barriers (`asm::dmb()`) prevent instruction reordering that would skew results.

### FPU Validation (RP2350)

The RP2350 benchmark verifies FPU is enabled by reading the `CPACR` register:

```
FPU Enabled: true
```

This confirms hardware floating-point is active.

## Understanding Results

### Cycle Counts

- Lower is better
- Includes full `update()` pipeline overhead
- Measured on real hardware, not simulation
