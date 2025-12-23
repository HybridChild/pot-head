use pot_head::hysteresis::{HysteresisMode, HysteresisState};

use core::marker::PhantomData;
#[cfg(feature = "hysteresis-schmitt")]
use pot_head::hysteresis::SchmittState;

#[test]
fn test_none_passes_through() {
    let mode: HysteresisMode<i32> = HysteresisMode::None(PhantomData);
    let mut state = HysteresisState::default();

    assert_eq!(mode.apply(100, &mut state), 100);
    assert_eq!(mode.apply(200, &mut state), 200);
    assert_eq!(mode.apply(50, &mut state), 50);
}

#[cfg(feature = "hysteresis-threshold")]
#[test]
fn test_change_threshold_ignores_small_changes() {
    let mode = HysteresisMode::ChangeThreshold { threshold: 10 };
    let mut state = HysteresisState {
        last_output: 100,
        #[cfg(feature = "hysteresis-schmitt")]
        schmitt_state: SchmittState::Low,
    };

    // Small changes should be ignored
    assert_eq!(mode.apply(105, &mut state), 100);
    assert_eq!(mode.apply(95, &mut state), 100);
    assert_eq!(mode.apply(109, &mut state), 100);

    // Large change should update
    assert_eq!(mode.apply(111, &mut state), 111);
    assert_eq!(state.last_output, 111);

    // Small changes from new value should be ignored
    assert_eq!(mode.apply(115, &mut state), 111);
    assert_eq!(mode.apply(107, &mut state), 111);

    // Large change downward should update
    assert_eq!(mode.apply(100, &mut state), 100);
    assert_eq!(state.last_output, 100);
}

#[cfg(feature = "hysteresis-threshold")]
#[test]
fn test_change_threshold_exact_boundary() {
    let mode = HysteresisMode::ChangeThreshold { threshold: 10 };
    let mut state = HysteresisState {
        last_output: 100,
        #[cfg(feature = "hysteresis-schmitt")]
        schmitt_state: SchmittState::Low,
    };

    // Exactly at threshold should NOT trigger (need to EXCEED threshold)
    assert_eq!(mode.apply(110, &mut state), 100);
    assert_eq!(mode.apply(90, &mut state), 100);

    // Just beyond threshold should trigger
    assert_eq!(mode.apply(111, &mut state), 111);
}

#[cfg(feature = "hysteresis-schmitt")]
#[test]
fn test_schmitt_trigger_basic() {
    let mode = HysteresisMode::SchmittTrigger {
        rising: 2050,
        falling: 2045,
    };
    let mut state = HysteresisState {
        last_output: 2045,
        schmitt_state: SchmittState::Low,
    };

    // Below rising threshold - should stay low
    assert_eq!(mode.apply(2048, &mut state), 2045);
    assert_eq!(state.schmitt_state, SchmittState::Low);

    // At rising threshold - should switch high
    assert_eq!(mode.apply(2050, &mut state), 2050);
    assert_eq!(state.schmitt_state, SchmittState::High);

    // Above falling threshold but below rising - should stay high
    assert_eq!(mode.apply(2047, &mut state), 2050);
    assert_eq!(state.schmitt_state, SchmittState::High);

    // At falling threshold - should switch low
    assert_eq!(mode.apply(2045, &mut state), 2045);
    assert_eq!(state.schmitt_state, SchmittState::Low);
}

#[cfg(feature = "hysteresis-schmitt")]
#[test]
fn test_schmitt_trigger_prevents_oscillation() {
    let mode = HysteresisMode::SchmittTrigger {
        rising: 100,
        falling: 90,
    };
    let mut state = HysteresisState {
        last_output: 90,
        schmitt_state: SchmittState::Low,
    };

    // Input oscillating around 95 - output should remain stable
    assert_eq!(mode.apply(96, &mut state), 90); // Start low
    assert_eq!(mode.apply(98, &mut state), 90); // Still low
    assert_eq!(mode.apply(92, &mut state), 90); // Still low
    assert_eq!(mode.apply(97, &mut state), 90); // Still low
    assert_eq!(state.schmitt_state, SchmittState::Low);

    // Only switches when crossing rising threshold
    assert_eq!(mode.apply(100, &mut state), 100);
    assert_eq!(state.schmitt_state, SchmittState::High);

    // Now oscillating stays high
    assert_eq!(mode.apply(96, &mut state), 100);
    assert_eq!(mode.apply(98, &mut state), 100);
    assert_eq!(mode.apply(92, &mut state), 100);
    assert_eq!(mode.apply(97, &mut state), 100);
    assert_eq!(state.schmitt_state, SchmittState::High);

    // Only switches when crossing falling threshold
    assert_eq!(mode.apply(90, &mut state), 90);
    assert_eq!(state.schmitt_state, SchmittState::Low);
}

#[cfg(feature = "hysteresis-schmitt")]
#[test]
fn test_schmitt_validation() {
    let valid = HysteresisMode::SchmittTrigger {
        rising: 100,
        falling: 90,
    };
    assert!(valid.validate().is_ok());

    let invalid = HysteresisMode::SchmittTrigger {
        rising: 90,
        falling: 100,
    };
    assert!(invalid.validate().is_err());

    let invalid_equal = HysteresisMode::SchmittTrigger {
        rising: 100,
        falling: 100,
    };
    assert!(invalid_equal.validate().is_err());
}

#[test]
fn test_none_validation() {
    let mode: HysteresisMode<i32> = HysteresisMode::None(PhantomData);
    assert!(mode.validate().is_ok());
}

#[cfg(feature = "hysteresis-threshold")]
#[test]
fn test_change_threshold_validation() {
    let mode = HysteresisMode::ChangeThreshold { threshold: 10 };
    assert!(mode.validate().is_ok());
}
