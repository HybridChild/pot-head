use pot_head::{Config, ConfigError, HysteresisMode, PotHead};

#[test]
fn test_invalid_input_range() {
    let config = Config {
        input_min: 100_u16,
        input_max: 100_u16,  // Same as min - invalid
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
    };

    let result = PotHead::new(config);
    assert!(matches!(result, Err(ConfigError::InvalidInputRange)));
}

#[test]
fn test_inverted_input_range() {
    let config = Config {
        input_min: 200_u16,
        input_max: 100_u16,  // Less than min - invalid
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
    };

    let result = PotHead::new(config);
    assert!(matches!(result, Err(ConfigError::InvalidInputRange)));
}

#[test]
fn test_invalid_output_range() {
    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 1.0_f32,
        output_max: 1.0_f32,  // Same as min - invalid
        hysteresis: HysteresisMode::none(),
    };

    let result = PotHead::new(config);
    assert!(matches!(result, Err(ConfigError::InvalidOutputRange)));
}
