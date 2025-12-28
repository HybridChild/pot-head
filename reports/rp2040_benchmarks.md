# RP2040 Benchmark Results

> **Note:** These are reference results from specific hardware. Performance may vary
> depending on your hardware revision, toolchain version, and environmental factors.
> Run the benchmark yourself for accurate results on your setup.

**Last Updated:** 2025-12-28 16:59:21  
**Toolchain:** rustc 1.91.1 (ed61e7d7e 2025-11-07)  
**Hardware:** Raspberry Pi Pico (RP2040)  
**Target:** thumbv6m-none-eabi (Cortex-M0+, no FPU)  
**Optimization:** --release (LTO enabled)

## Results

```
    Finished `release` profile [optimized + debuginfo] target(s) in 0.05s
     Running `probe-rs run --chip RP2040 --no-timestamps target/thumbv6m-none-eabi/release/pot-head-bench-rp2040`
     Finished in 1.79s

pot-head Hardware Benchmark
===========================

Platform: RP2040 (Cortex-M0+)
CPU: 125 MHz, FPU: None (soft-float)
Target: thumbv6m-none-eabi

PotHead::update() Performance
-----------------------------

Scenario              Cycles      µs
==================  ========  ======
Baseline                1123    8.99
With EMA                1590   12.72
With Log Curve          4065   32.52
Full Featured           5877   47.02
MA Window=4             1923   15.39
MA Window=16            3342   26.74
u16→u16                 1796   14.37

```
