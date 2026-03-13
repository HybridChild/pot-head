# pot-head sizeof Report

> **Note:** These are reference results for a specific target architecture. Struct sizes
> are deterministic and depend only on the target architecture and compiler version,
> not on hardware. Results should be identical across machines with the same toolchain.

**Generated:** 2026-03-13 07:14:31 UTC

**Target:** ARM Cortex-M4F/M7 (with FPU) (`thumbv7em-none-eabihf`)

## Component Sizes

| Type | Size (bytes) |
|------|--------------|
| Config<u16, f32> | 44 |
| Config<u16, u16> | 40 |
| Config<u8, f32> | 40 |
| State<f32> | 184 |
| PotHead<u16, f32> | 188 |
| PotHead<u16, u16> | 188 |
| PotHead<u8, f32> | 188 |

## Common Configurations

Real-world embedded usage scenarios:

| Use Case | Type Combo | Est. Total RAM |
|----------|------------|----------------|
| Simple volume knob | u16→f32 | ~188 bytes |
| Integer scaler | u16→u16 | ~188 bytes |
| Compact config | u8→f32 | ~188 bytes |

## Summary

- **Config storage** (flash): 44-44 bytes (depends on type parameters)
- **Runtime state** (RAM): 184 bytes per instance
- **Typical usage**: ~188 bytes total per potentiometer

These measurements are for **ARM Cortex-M4F/M7 (with FPU)** and represent actual embedded target sizes.

---

*Run `tools/sizeof-calculator/generate-report.sh` to regenerate this report.*
*Optionally specify a target: `generate-report.sh thumbv6m-none-eabi`*
