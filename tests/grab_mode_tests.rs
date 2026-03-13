#![cfg(feature = "grab-mode")]

use pot_head::{
    Config, GrabMode, HysteresisMode, NoiseFilter, PotHead, ResponseCurve, SnapZone, SnapZoneType,
};

static CONFIG_NONE: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 1000,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::None(core::marker::PhantomData),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &[],
    grab_mode: GrabMode::None,
};

static CONFIG_PICKUP: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 1000,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::None(core::marker::PhantomData),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &[],
    grab_mode: GrabMode::Pickup,
};

static CONFIG_PASSTHROUGH: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 1000,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::None(core::marker::PhantomData),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &[],
    grab_mode: GrabMode::PassThrough,
};

static CONFIG_PASSTHROUGH_SNAPPED: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 1000,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::None(core::marker::PhantomData),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &[
        SnapZone::new(0.5, 0.05, SnapZoneType::Snap), // snap to center
    ],
    grab_mode: GrabMode::PassThrough,
};

fn create_test_config(grab_mode: GrabMode) -> &'static Config<u16, f32> {
    match grab_mode {
        GrabMode::None => &CONFIG_NONE,
        GrabMode::Pickup => &CONFIG_PICKUP,
        GrabMode::PassThrough => &CONFIG_PASSTHROUGH,
    }
}

#[test]
fn test_grab_mode_none() {
    let config = create_test_config(GrabMode::None);
    let mut pot = PotHead::new(&config).unwrap();

    // No grab mode - output follows input immediately
    assert_eq!(pot.process(500), 0.5);
    assert_eq!(pot.process(200), 0.2);
    assert_eq!(pot.process(800), 0.8);

    // Always grabbed in None mode
    assert!(!pot.is_waiting_for_grab());
}

#[test]
fn test_pickup_mode_basic() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    // Set virtual value to 70%
    pot.set_virtual_value(0.7);

    // Pot is at 20% (below virtual value)
    assert!(pot.is_waiting_for_grab());
    assert_eq!(pot.process(200), 0.7); // Locked at virtual value

    // Move pot but still below virtual value
    assert_eq!(pot.process(300), 0.7); // Still locked
    assert_eq!(pot.process(500), 0.7); // Still locked

    // Cross virtual value (70% = 700)
    assert_eq!(pot.process(700), 0.7); // Grabbed!
    assert!(!pot.is_waiting_for_grab());

    // Now follows pot position
    assert_eq!(pot.process(800), 0.8);
    assert_eq!(pot.process(900), 0.9);
}

#[test]
fn test_pickup_mode_already_above() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    // Set virtual value to 30%
    pot.set_virtual_value(0.3);

    // Pot starts at 80% (already above virtual value)
    assert_eq!(pot.process(800), 0.8); // Immediately grabbed!
    assert!(!pot.is_waiting_for_grab());

    // Continues to follow
    assert_eq!(pot.process(600), 0.6);
}

#[test]
fn test_passthrough_mode_from_below() {
    let config = create_test_config(GrabMode::PassThrough);
    let mut pot = PotHead::new(&config).unwrap();

    // Set virtual value to 70%
    pot.set_virtual_value(0.7);

    // Pot is at 20% (below virtual value)
    assert!(pot.is_waiting_for_grab());
    assert_eq!(pot.process(200), 0.7); // Locked

    // Move upward but not crossing yet
    assert_eq!(pot.process(500), 0.7); // Still locked

    // Cross virtual value from below
    assert_eq!(pot.process(700), 0.7); // Grabbed!
    assert!(!pot.is_waiting_for_grab());

    // Now follows
    assert_eq!(pot.process(800), 0.8);
}

#[test]
fn test_passthrough_mode_from_above() {
    let config = create_test_config(GrabMode::PassThrough);
    let mut pot = PotHead::new(&config).unwrap();

    // Set virtual value to 30%
    pot.set_virtual_value(0.3);

    // Pot is at 80% (above virtual value)
    assert!(pot.is_waiting_for_grab());
    assert_eq!(pot.process(800), 0.3); // Locked

    // Move downward but not crossing yet
    assert_eq!(pot.process(500), 0.3); // Still locked

    // Cross virtual value from above
    assert_eq!(pot.process(300), 0.3); // Grabbed!
    assert!(!pot.is_waiting_for_grab());

    // Now follows
    assert_eq!(pot.process(200), 0.2);
}

#[test]
fn test_passthrough_mode_bidirectional() {
    let config = create_test_config(GrabMode::PassThrough);
    let mut pot = PotHead::new(&config).unwrap();

    // Set virtual value to 50%
    pot.set_virtual_value(0.5);

    // Test crossing from below
    pot.set_virtual_value(0.5);
    assert_eq!(pot.process(200), 0.5); // Below, locked
    assert_eq!(pot.process(500), 0.5); // Grabbed from below
    assert!(!pot.is_waiting_for_grab());

    // Reset for next test
    pot.set_virtual_value(0.5);

    // Test crossing from above
    assert_eq!(pot.process(800), 0.5); // Above, locked
    assert_eq!(pot.process(500), 0.5); // Grabbed from above
    assert!(!pot.is_waiting_for_grab());
}

#[test]
fn test_physical_position_tracking() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    // Set virtual value to 70%
    pot.set_virtual_value(0.7);

    // Move pot to 30%
    pot.process(300);
    assert_eq!(pot.current_output(), 0.7); // Virtual locked at 70%
    assert_eq!(pot.physical_position(), 0.3); // Physical at 30%

    // Move pot to 50%
    pot.process(500);
    assert_eq!(pot.current_output(), 0.7); // Still locked
    assert_eq!(pot.physical_position(), 0.5); // Physical at 50%

    // Grab at 70%
    pot.process(700);
    assert_eq!(pot.current_output(), 0.7); // Now matches
    assert_eq!(pot.physical_position(), 0.7);
}

#[test]
fn test_set_virtual_value_resets_grab() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    // Grab pot at 50%
    pot.process(500);
    assert!(!pot.is_waiting_for_grab());

    // Simulate automation/preset change
    pot.set_virtual_value(0.8);
    assert!(pot.is_waiting_for_grab()); // Now waiting for grab again

    // Pot still at 50%, needs to move to 80%
    assert_eq!(pot.process(500), 0.8); // Locked
    assert_eq!(pot.process(600), 0.8); // Still locked
    assert_eq!(pot.process(800), 0.8); // Grabbed!
    assert!(!pot.is_waiting_for_grab());
}

#[test]
fn test_pickup_mode_no_overshoot_required() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    pot.set_virtual_value(0.7);

    // Move exactly to virtual value
    assert_eq!(pot.process(700), 0.7); // Should grab at exact value
    assert!(!pot.is_waiting_for_grab());
}

#[test]
fn test_passthrough_exact_crossing() {
    let config = create_test_config(GrabMode::PassThrough);
    let mut pot = PotHead::new(&config).unwrap();

    pot.set_virtual_value(0.5);

    // Move from below to exactly virtual value
    pot.process(300); // Below
    assert_eq!(pot.process(500), 0.5); // Exact crossing
    assert!(!pot.is_waiting_for_grab());
}

#[test]
fn test_grab_mode_helpers() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    pot.set_virtual_value(0.7);
    pot.process(300);

    // Waiting for grab
    assert!(pot.is_waiting_for_grab());
    assert_eq!(pot.current_output(), 0.7);
    assert_eq!(pot.physical_position(), 0.3);

    // Grab
    pot.process(700);
    assert!(!pot.is_waiting_for_grab());
    assert_eq!(pot.current_output(), 0.7);
    assert_eq!(pot.physical_position(), 0.7);
}

#[test]
fn test_release_method() {
    let config = create_test_config(GrabMode::Pickup);
    let mut pot = PotHead::new(&config).unwrap();

    // Grab pot at 60%
    pot.process(600);
    assert!(!pot.is_waiting_for_grab());
    assert_eq!(pot.current_output(), 0.6);

    // Move pot to 40%
    pot.process(400);
    assert_eq!(pot.current_output(), 0.4);

    // Release - should ungrab and set virtual to current physical position
    pot.detach();
    assert!(pot.is_waiting_for_grab()); // Now waiting for grab
    assert_eq!(pot.current_output(), 0.4); // Virtual value set to last physical
    assert_eq!(pot.physical_position(), 0.4);

    // Pot is still at 40%, so output stays locked
    assert_eq!(pot.process(400), 0.4);

    // Move pot to re-grab
    assert_eq!(pot.process(600), 0.6); // Grabbed at 60%
    assert!(!pot.is_waiting_for_grab());
}

#[test]
fn test_release_for_mode_switching() {
    // Simulate switching between volume and backlight control
    let volume_config = create_test_config(GrabMode::Pickup);
    let backlight_config = create_test_config(GrabMode::Pickup);

    let mut volume_pot = PotHead::new(&volume_config).unwrap();
    let mut backlight_pot = PotHead::new(&backlight_config).unwrap();

    // Volume mode: pot grabbed at 50%
    volume_pot.process(500);
    assert_eq!(volume_pot.current_output(), 0.5);
    assert!(!volume_pot.is_waiting_for_grab());

    // Move to 70%
    volume_pot.process(700);
    assert_eq!(volume_pot.current_output(), 0.7);

    // Switch to backlight mode - release volume
    volume_pot.detach();
    assert!(volume_pot.is_waiting_for_grab());
    assert_eq!(volume_pot.current_output(), 0.7); // Virtual stays at last physical

    // Backlight pot starts fresh - for mode switching, you'd typically
    // want to initialize it to current pot position to avoid jumps
    // Option 1: Let it grab naturally (might cause jump)
    backlight_pot.process(700); // Pot at 70%, backlight grabs
    assert_eq!(backlight_pot.current_output(), 0.7);

    // Now backlight follows the pot
    assert_eq!(backlight_pot.process(600), 0.6);
}

#[test]
fn test_detach_preserves_snapped_virtual_value() {
    // Regression test: detach() must store last_output (post-snap), not physical_position
    // (pre-snap). If it stores physical_position, the virtual value after detach is the raw
    // ADC position inside the snap zone rather than the snapped output, causing the held
    // value to drift (e.g. 0.48 instead of 0.5) when the user returns to this parameter.
    let mut pot = PotHead::new(&CONFIG_PASSTHROUGH_SNAPPED).unwrap();

    // Simulate a parameter already set to the snap target (0.5).
    pot.set_virtual_value(0.5);

    // Pot starts below virtual value; first process initialises passthrough state.
    pot.process(300);
    // Cross the virtual value from below — physical 480 is inside the snap zone,
    // so snapped = 0.5, which equals virtual_value → grabbed.
    pot.process(480);
    assert!(!pot.is_waiting_for_grab());
    assert_eq!(pot.current_output(), 0.5);

    // Physical pot drifts slightly inside the snap zone (530 → snapped to 0.5).
    // physical_position ≈ 0.53, but last_output == 0.5.
    pot.process(530);
    assert_eq!(pot.current_output(), 0.5);

    // Detach — pot is grabbed; physical_position is ~0.53, last_output is 0.5.
    pot.detach();
    assert!(pot.is_waiting_for_grab());
    // virtual_value must be the snapped output (0.5), not the raw physical position (~0.53).
    assert_eq!(pot.current_output(), 0.5);
}
