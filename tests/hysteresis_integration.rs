use pot_head::{Config, HysteresisMode, NoiseFilter, PotHead, ResponseCurve};

#[test]
fn test_pothead_with_no_hysteresis() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Start at midpoint
    let output1 = pot.update(2048);
    assert!((output1 - 0.5).abs() < 0.001);

    // Small change should pass through (no hysteresis blocking it)
    let output2 = pot.update(2049);
    assert!(output2 > output1, "Small changes should pass through with no hysteresis");

    // Another small change should also pass through
    let output3 = pot.update(2050);
    assert!(output3 > output2, "Every change should update with no hysteresis");
}

#[cfg(feature = "hysteresis-threshold")]
#[test]
fn test_pothead_with_change_threshold() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.05 }, // 5% threshold in normalized space
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Initial value
    let initial = pot.update(2048); // ~50%
    assert!((initial - 0.5).abs() < 0.001);

    // Small change (< 5%) should be ignored
    let output = pot.update(2150); // ~52.5% - change of 2.5%
    assert_eq!(output, initial, "Small change should be ignored");

    // Large change (> 5%) should update
    // Need 5% of 4095 = ~205 units change
    let output = pot.update(2300); // ~56.2% - change of 6.2%
    assert!(output != initial, "Output should have changed");
    assert!((output - 0.562).abs() < 0.001);

    // Small change from new value should be ignored
    let prev_output = output;
    let output = pot.update(2400); // ~58.6% - change of 2.4%
    assert_eq!(output, prev_output, "Small change should be ignored");

    // Another large change should update
    let output = pot.update(2800); // ~68.4% - change of 12.2%
    assert!(output != prev_output, "Output should have changed");
    assert!((output - 0.684).abs() < 0.001);
}

#[cfg(feature = "hysteresis-schmitt")]
#[test]
fn test_pothead_with_schmitt_trigger() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        hysteresis: HysteresisMode::SchmittTrigger {
            rising: 0.6,
            falling: 0.4,
        },
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Start below falling threshold - should output falling value
    let output = pot.update(1000); // ~24%
    assert_eq!(output, 0.4);

    // Move between thresholds - should stay at falling value
    let output = pot.update(2048); // ~50%
    assert_eq!(output, 0.4);

    // Cross rising threshold - should switch to rising value
    let output = pot.update(2700); // ~66%
    assert_eq!(output, 0.6);

    // Move between thresholds - should stay at rising value
    let output = pot.update(2048); // ~50%
    assert_eq!(output, 0.6);

    // Cross falling threshold - should switch to falling value
    let output = pot.update(1500); // ~37%
    assert_eq!(output, 0.4);
}

#[cfg(feature = "hysteresis-threshold")]
#[test]
fn test_hysteresis_with_different_types() {
    // Test u8 -> i16
    let config = Config {
        input_min: 0_u8,
        input_max: 255_u8,
        output_min: -100_i16,
        output_max: 100_i16,
        hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.1 }, // 10% threshold
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Initial value at center
    let output = pot.update(127); // ~50%
    assert_eq!(output, 0);

    // Small change should be ignored
    let output = pot.update(140); // ~55%
    assert_eq!(output, 0);

    // Large change should update
    let output = pot.update(180); // ~70%
    assert_ne!(output, 0);
    assert!((output - 40).abs() < 2); // ~40
}

#[cfg(feature = "hysteresis-schmitt")]
#[test]
fn test_invalid_schmitt_config() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        hysteresis: HysteresisMode::SchmittTrigger {
            rising: 0.4,  // Invalid: rising <= falling
            falling: 0.6,
        },
    };

    // Should fail validation
    let result = PotHead::new(config);
    assert!(result.is_err());
}
