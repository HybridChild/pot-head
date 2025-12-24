# pot-head

Embedded developers working with physical controls (potentiometers, faders, sliders) face common challenges:
- **Noisy ADC readings** causing jittery output
- **Parameter jumps** when physical position doesn't match virtual state
- **Lack of professional polish** (no snap zones, no smooth response curves)
- **Boilerplate code** reimplemented in every project

This crate provides a `no_std` library for processing potentiometer inputs in embedded systems. It transforms raw ADC values into clean, processed output values through configurable filters, curves, and response modes.

## Core Principle

**pot-head is a pure mathematical abstraction.** It transforms raw input values (typically ADC readings) into processed output values based on configuration and internal state. The crate handles no I/O, no interrupts, no HAL integration - just math.

```
Raw ADC Value → pot-head Processing → Clean Output Value
```

## Target Use Cases

- **Audio equipment**: Mixers, synthesizers, effects processors (parameter automation with fetch/grab mode)
- **Industrial control panels**: Machine interfaces requiring noise immunity and reliability
- **Consumer devices**: Any embedded system with physical controls for human interaction
