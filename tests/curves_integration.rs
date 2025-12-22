use pot_head::{Config, HysteresisMode, PotHead, ResponseCurve};

#[test]
fn test_linear_curve_integration() {
    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Linear curve should map directly
    assert_eq!(pot.update(0), 0.0);
    assert_eq!(pot.update(50), 0.5);
    assert_eq!(pot.update(100), 1.0);
}

#[cfg(feature = "log-curve")]
#[test]
fn test_logarithmic_curve_integration() {
    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Logarithmic,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Logarithmic curve boundaries
    let min_output = pot.update(0);
    let max_output = pot.update(100);

    assert!((min_output - 0.0).abs() < 0.001, "Min should be ~0.0, got {}", min_output);
    assert!((max_output - 1.0).abs() < 0.001, "Max should be ~1.0, got {}", max_output);

    // Logarithmic curve should compress lower values
    let quarter_output = pot.update(25);
    assert!(quarter_output < 0.15, "Quarter should be <0.2 with log curve, got {}", quarter_output);

    // Middle should be less than 0.5 (shifted down)
    let mid_output = pot.update(50);
    assert!(mid_output < 0.5, "Middle should be <0.5 with log curve, got {}", mid_output);

    // Should be monotonically increasing
    assert!(min_output < quarter_output);
    assert!(quarter_output < mid_output);
    assert!(mid_output < max_output);
}
