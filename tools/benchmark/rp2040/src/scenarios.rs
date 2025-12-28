use pot_head::*;

// Scenario 1: Baseline (minimal processing)
pub static BASELINE: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    hysteresis: HysteresisMode::none(),
    snap_zones: &[],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::None,
};

// Scenario 2: With EMA filter
pub static WITH_EMA: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::none(),
    snap_zones: &[],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::None,
};

// Scenario 3: With logarithmic curve (shows FPU benefit)
#[cfg(feature = "std-math")]
pub static WITH_LOG_CURVE: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Logarithmic,
    filter: NoiseFilter::None,
    hysteresis: HysteresisMode::none(),
    snap_zones: &[],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::None,
};

// Scenario 4: Full featured (worst case)
pub static FULL_FEATURED: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    #[cfg(feature = "std-math")]
    curve: ResponseCurve::Logarithmic,
    #[cfg(not(feature = "std-math"))]
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.01 },
    snap_zones: &[
        SnapZone::new(0.0, 0.02, SnapZoneType::Snap),
        SnapZone::new(0.5, 0.05, SnapZoneType::Snap),
        SnapZone::new(1.0, 0.02, SnapZoneType::Snap),
    ],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::Pickup,
};

// Scenario 5: MovingAverage filter (window=4)
#[cfg(feature = "moving-average")]
pub static MA_WINDOW_4: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::MovingAverage { window_size: 4 },
    hysteresis: HysteresisMode::none(),
    snap_zones: &[],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::None,
};

// Scenario 6: MovingAverage filter (window=16)
#[cfg(feature = "moving-average")]
pub static MA_WINDOW_16: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::MovingAverage { window_size: 16 },
    hysteresis: HysteresisMode::none(),
    snap_zones: &[],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::None,
};

// Scenario 7: Type comparison (u16→u16 vs u16→f32)
pub static U16_TO_U16: Config<u16, u16> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 1000,
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    hysteresis: HysteresisMode::none(),
    snap_zones: &[],
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::None,
};
