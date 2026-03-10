# RP2350 Benchmark Results

> **Note:** These are reference results from specific hardware. Performance may vary
> depending on your hardware revision, toolchain version, and environmental factors.
> Run the benchmark yourself for accurate results on your setup.

**Last Updated:** 2025-12-28 16:51:35  
**Toolchain:** rustc 1.91.1 (ed61e7d7e 2025-11-07)  
**Hardware:** Raspberry Pi Pico 2 (RP2350)  
**Target:** thumbv8m.main-none-eabihf (Cortex-M33F with FPU)  
**Optimization:** --release (LTO enabled)

## Results

```
    Finished `release` profile [optimized + debuginfo] target(s) in 0.06s
     Running `probe-rs run --chip RP235x --no-timestamps target/thumbv8m.main-none-eabihf/release/pot-head-bench-rp2350`
     Finished in 1.92s

pot-head Hardware Benchmark
===========================

Platform: RP2350 (Cortex-M33F)
CPU: 150 MHz, FPU: Single-precision (hard-float)
Target: thumbv8m.main-none-eabihf
FPU Enabled: true

PotHead::process() Performance
-----------------------------

Scenario              Cycles      µs
==================  ========  ======
Baseline                 128    0.86
With EMA                 150    1.00
With Log Curve           264    1.76
Full Featured            341    2.28
MA Window=4              195    1.30
MA Window=16             235    1.57
u16→u16                  147    0.98

```
