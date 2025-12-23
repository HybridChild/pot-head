use pot_head::{Config, GrabMode, HysteresisMode, NoiseFilter, PotHead, ResponseCurve, SnapZone};

#[cfg(any(feature = "snap-zone-snap", feature = "snap-zone-dead"))]
use pot_head::SnapZoneType;

#[cfg(feature = "snap-zone-snap")]
#[test]
fn test_snap_zone_basic() {
    static SNAP_ZONES: [SnapZone<f32>; 1] = [
        SnapZone::new(0.0, 0.05, SnapZoneType::Snap), // Snap to 0% within ±5%
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &SNAP_ZONES,
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Within snap zone - should snap to 0.0
    let result = pot.update(2); // 2% of range
    assert_eq!(result, 0.0, "Should snap to 0.0");

    let result = pot.update(5); // 5% of range
    assert_eq!(result, 0.0, "Should snap to 0.0 at boundary");

    // Outside snap zone - should pass through
    let result = pot.update(10); // 10% of range
    assert!(
        (result - 0.1).abs() < 0.01,
        "Should be ~0.1, got {}",
        result
    );
}

#[cfg(feature = "snap-zone-snap")]
#[test]
fn test_multiple_snap_zones() {
    static SNAP_ZONES: [SnapZone<f32>; 3] = [
        SnapZone::new(0.0, 0.02, SnapZoneType::Snap), // 0% ±2%
        SnapZone::new(0.5, 0.03, SnapZoneType::Snap), // 50% ±3%
        SnapZone::new(1.0, 0.02, SnapZoneType::Snap), // 100% ±2%
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &SNAP_ZONES,
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Test each snap zone
    assert_eq!(pot.update(1), 0.0); // Snap to 0%
    assert_eq!(pot.update(50), 0.5); // Snap to 50%
    assert_eq!(pot.update(99), 1.0); // Snap to 100%

    // Test between zones - no snapping
    let result = pot.update(25);
    assert!(
        (result - 0.25).abs() < 0.01,
        "Should be ~0.25, got {}",
        result
    );
}

#[cfg(feature = "snap-zone-dead")]
#[test]
fn test_dead_zone_basic() {
    static SNAP_ZONES: [SnapZone<f32>; 1] = [
        SnapZone::new(0.5, 0.05, SnapZoneType::Dead), // Dead zone at 50% ±5%
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &SNAP_ZONES,
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Move to 25% (outside dead zone)
    let result = pot.update(25);
    assert!(
        (result - 0.25).abs() < 0.01,
        "Should be ~0.25, got {}",
        result
    );

    // Enter dead zone - output should freeze at last value (0.25)
    let result = pot.update(50);
    assert!(
        (result - 0.25).abs() < 0.01,
        "Dead zone should hold last output (0.25), got {}",
        result
    );

    // Still in dead zone - should still be frozen
    let result = pot.update(52);
    assert!(
        (result - 0.25).abs() < 0.01,
        "Dead zone should hold last output (0.25), got {}",
        result
    );

    // Exit dead zone - should update again
    let result = pot.update(60);
    assert!(
        (result - 0.6).abs() < 0.01,
        "Should be ~0.6 after exiting dead zone, got {}",
        result
    );
}

#[cfg(all(feature = "snap-zone-snap", feature = "snap-zone-dead"))]
#[test]
fn test_mixed_snap_and_dead_zones() {
    static SNAP_ZONES: [SnapZone<f32>; 2] = [
        SnapZone::new(0.0, 0.05, SnapZoneType::Snap), // Snap to 0%
        SnapZone::new(0.5, 0.05, SnapZoneType::Dead), // Dead zone at 50%
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &SNAP_ZONES,
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Snap zone behavior
    assert_eq!(pot.update(3), 0.0, "Should snap to 0.0");

    // Move outside snap zone
    let result = pot.update(25);
    assert!(
        (result - 0.25).abs() < 0.01,
        "Should be ~0.25, got {}",
        result
    );

    // Enter dead zone - should freeze at 0.25
    let result = pot.update(50);
    assert!(
        (result - 0.25).abs() < 0.01,
        "Dead zone should hold 0.25, got {}",
        result
    );
}

#[cfg(feature = "snap-zone-snap")]
#[test]
fn test_overlapping_zones_first_match_wins() {
    // Intentional overlap: second zone is within first zone's range
    static SNAP_ZONES: [SnapZone<f32>; 2] = [
        SnapZone::new(0.0, 0.1, SnapZoneType::Snap), // 0% ±10% (range: -0.1 to 0.1)
        SnapZone::new(0.05, 0.03, SnapZoneType::Snap), // 5% ±3% (range: 0.02 to 0.08, overlaps!)
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &SNAP_ZONES,
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Input at 5% - matches both zones, but first one wins
    let result = pot.update(5);
    assert_eq!(
        result, 0.0,
        "First zone should win (snap to 0.0), got {}",
        result
    );
}

#[test]
fn test_empty_snap_zones() {
    static SNAP_ZONES: [SnapZone<f32>; 0] = [];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &SNAP_ZONES,
        grab_mode: GrabMode::None,
    };

    let mut pot = PotHead::new(config).unwrap();

    // Should pass through unchanged
    assert_eq!(pot.update(0), 0.0);
    assert_eq!(pot.update(50), 0.5);
    assert_eq!(pot.update(100), 1.0);
}

#[cfg(feature = "snap-zone-snap")]
#[test]
fn test_snap_zone_validation_overlaps() {
    static OVERLAPPING_ZONES: [SnapZone<f32>; 2] = [
        SnapZone::new(0.0, 0.1, SnapZoneType::Snap),
        SnapZone::new(0.05, 0.1, SnapZoneType::Snap), // Overlaps with first
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &OVERLAPPING_ZONES,
        grab_mode: GrabMode::None,
    };

    // Config is valid - overlaps are allowed by default
    assert!(config.validate().is_ok());

    // But optional validation should catch overlap
    assert!(config.validate_snap_zones().is_err());
}

#[cfg(feature = "snap-zone-snap")]
#[test]
fn test_snap_zone_validation_no_overlaps() {
    static NON_OVERLAPPING_ZONES: [SnapZone<f32>; 3] = [
        SnapZone::new(0.0, 0.02, SnapZoneType::Snap),
        SnapZone::new(0.5, 0.03, SnapZoneType::Snap),
        SnapZone::new(1.0, 0.02, SnapZoneType::Snap),
    ];

    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &NON_OVERLAPPING_ZONES,
        grab_mode: GrabMode::None,
    };

    // Should pass both validations
    assert!(config.validate().is_ok());
    assert!(config.validate_snap_zones().is_ok());
}
