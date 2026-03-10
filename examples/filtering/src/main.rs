//! Demonstrates noise filtering with pot-head
//!
//! This example shows how to use EMA and Moving Average filters
//! to smooth noisy ADC readings.

use pot_head::{Config, GrabMode, HysteresisMode, NoiseFilter, PotHead, ResponseCurve, SnapZone};

static EMPTY_SNAP_ZONES: [SnapZone<f32>; 0] = [];

static NO_FILTER_CONFIG: Config<u16, u16> = Config {
    input_min: 0_u16,
    input_max: 4095_u16,
    output_min: 0_u16,
    output_max: 4095_u16,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static MA_FILTER_CONFIG: Config<u16, u16> = Config {
    input_min: 0_u16,
    input_max: 4095_u16,
    output_min: 0_u16,
    output_max: 4095_u16,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::MovingAverage { window_size: 5 },
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static EMA_FILTER_CONFIG: Config<u16, u16> = Config {
    input_min: 0_u16,
    input_max: 4095_u16,
    output_min: 0_u16,
    output_max: 4095_u16,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static FILTER_HYSTERESIS_CONFIG: Config<u16, u16> = Config {
    input_min: 0_u16,
    input_max: 4095_u16,
    output_min: 0_u16,
    output_max: 4095_u16,
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.01 },
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

fn main() {
    println!("=== pot-head Filtering Examples ===\n");

    let noisy_samples = [2048, 2100, 2000, 2080, 1990, 2050, 2020, 2060];

    // Example 1: No Filter (for comparison)
    println!("1. No Filter (raw passthrough)");
    let mut pot = PotHead::new(&NO_FILTER_CONFIG).expect("Valid config");

    println!("   Input → Output");
    for &sample in &noisy_samples {
        let output = pot.process(sample);
        println!("   {:4} → {:4}", sample, output);
    }
    println!();

    // Example 2: Moving Average Filter
    println!("2. Moving Average Filter (window=5)");
    let mut pot = PotHead::new(&MA_FILTER_CONFIG).expect("Valid config");

    // Simulate noisy readings
    println!("   Input → Output (filtered)");
    for &sample in &noisy_samples {
        let output = pot.process(sample);
        println!("   {:4} → {:4}", sample, output);
    }
    println!();

    // Example 3: EMA Filter (Exponential Moving Average)
    println!("3. EMA Filter (alpha=0.3)");
    let mut pot = PotHead::new(&EMA_FILTER_CONFIG).expect("Valid config");

    // Simulate noisy readings around 2048 (50%)
    println!("   Input → Output (filtered)");
    for &sample in &noisy_samples {
        let output = pot.process(sample);
        println!("   {:4} → {:4}", sample, output);
    }
    println!();

    // Example 4: Filter + Hysteresis
    println!("4. EMA Filter + 1% Threshold hysteresis");
    let mut pot = PotHead::new(&FILTER_HYSTERESIS_CONFIG).expect("Valid config");

    println!("   Combining smooth filtering with change threshold");
    println!("   Small changes ignored, large changes smoothed");
    println!("   Input → Output");

    for &sample in &noisy_samples {
        let output = pot.process(sample);
        println!("   {:4} → {:4}", sample, output);
    }
}
