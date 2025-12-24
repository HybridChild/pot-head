use pot_head::{Config, HysteresisMode, NoiseFilter, PotHead, ResponseCurve, SnapZone};

#[cfg(feature = "grab-mode")]
use pot_head::GrabMode;

static EMPTY_SNAP_ZONES: [SnapZone<f32>; 0] = [];

#[test]
fn test_u16_to_u16_normalization() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0_u16,
        output_max: 255_u16,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &EMPTY_SNAP_ZONES,
        #[cfg(feature = "grab-mode")]
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Test minimum
    assert_eq!(pot.update(0), 0);

    // Test maximum
    assert_eq!(pot.update(4095), 255);

    // Test middle (should be approximately 127-128)
    let mid = pot.update(2047);
    assert!(mid >= 127 && mid <= 128, "Middle value was {}", mid);

    // Test quarter point
    let quarter = pot.update(1023);
    assert!(
        quarter >= 63 && quarter <= 64,
        "Quarter value was {}",
        quarter
    );
}

#[test]
fn test_u16_to_f32_normalization() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &EMPTY_SNAP_ZONES,
        #[cfg(feature = "grab-mode")]
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Test minimum
    assert_eq!(pot.update(0), 0.0);

    // Test maximum
    assert_eq!(pot.update(4095), 1.0);

    // Test middle
    let mid = pot.update(2047);
    assert!((mid - 0.5).abs() < 0.01, "Middle value was {}", mid);

    // Test quarter
    let quarter = pot.update(1023);
    assert!(
        (quarter - 0.25).abs() < 0.01,
        "Quarter value was {}",
        quarter
    );
}

#[test]
fn test_input_clamping() {
    let config = Config {
        input_min: 100_u16,
        input_max: 200_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &EMPTY_SNAP_ZONES,
        #[cfg(feature = "grab-mode")]
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Test below minimum gets clamped
    assert_eq!(pot.update(50), 0.0);

    // Test above maximum gets clamped
    assert_eq!(pot.update(300), 1.0);

    // Test within range
    assert_eq!(pot.update(150), 0.5);
}

#[test]
fn test_inverted_output_range() {
    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 1.0_f32,
        output_max: 0.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &EMPTY_SNAP_ZONES,
        #[cfg(feature = "grab-mode")]
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // When input is minimum, output should be maximum (inverted)
    assert_eq!(pot.update(0), 1.0);

    // When input is maximum, output should be minimum (inverted)
    assert_eq!(pot.update(100), 0.0);

    // Middle should be 0.5
    let mid = pot.update(50);
    assert!((mid - 0.5).abs() < 0.01, "Middle value was {}", mid);
}

#[test]
fn test_same_type_conversion() {
    let config = Config {
        input_min: 0_f32,
        input_max: 1.0_f32,
        output_min: 0.0_f32,
        output_max: 100.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &EMPTY_SNAP_ZONES,
        #[cfg(feature = "grab-mode")]
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    assert_eq!(pot.update(0.0), 0.0);
    assert_eq!(pot.update(1.0), 100.0);
    assert_eq!(pot.update(0.5), 50.0);
}
