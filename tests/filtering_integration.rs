use pot_head::{Config, HysteresisMode, NoiseFilter, PotHead, ResponseCurve, SnapZone};

static EMPTY_SNAP_ZONES: [SnapZone<f32>; 0] = [];

#[cfg(feature = "filter-ema")]
#[test]
fn test_pothead_with_ema_filter() {
    let config = Config {
        input_min: 0_u16,
        input_max: 4095_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
        snap_zones: &EMPTY_SNAP_ZONES,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // First value initializes filter
    let out1 = pot.update(0);
    assert_eq!(out1, 0.0);

    // Step to max - should be smoothed
    let out2 = pot.update(4095);
    // With alpha=0.3: 0.3 * 1.0 + 0.7 * 0.0 = 0.3
    assert!((out2 - 0.3).abs() < 0.001, "Expected ~0.3, got {}", out2);

    // Another step - continues smoothing
    let out3 = pot.update(4095);
    // 0.3 * 1.0 + 0.7 * 0.3 = 0.51
    assert!((out3 - 0.51).abs() < 0.01, "Expected ~0.51, got {}", out3);
}

#[cfg(feature = "filter-moving-avg")]
#[test]
fn test_pothead_with_moving_average_filter() {
    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::MovingAverage { window_size: 3 },
        snap_zones: &EMPTY_SNAP_ZONES,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Build up moving average
    assert_eq!(pot.update(0), 0.0);    // [0.0] avg = 0.0
    assert_eq!(pot.update(30), 0.15);  // [0.0, 0.3] avg = 0.15
    assert_eq!(pot.update(60), 0.3);   // [0.0, 0.3, 0.6] avg = 0.3

    // Window slides
    let out = pot.update(90);  // [0.9, 0.3, 0.6] avg = 0.6
    assert!((out - 0.6).abs() < 0.001, "Expected 0.6, got {}", out);
}

#[cfg(feature = "filter-ema")]
#[test]
fn test_filter_smooths_noisy_input() {
    let config = Config {
        input_min: 0_u16,
        input_max: 1000_u16,
        output_min: 0_u16,
        output_max: 1000_u16,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.2 },
        snap_zones: &EMPTY_SNAP_ZONES,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Noisy readings around 500
    let noisy_samples = [500, 510, 490, 505, 495, 500, 498, 502];
    let mut outputs = Vec::new();

    for &sample in &noisy_samples {
        outputs.push(pot.update(sample));
    }

    // Output should be less noisy than input
    // Calculate variance of last few outputs
    let output_slice = &outputs[outputs.len() - 4..];
    let mean: f32 = output_slice.iter().map(|&x| x as f32).sum::<f32>() / 4.0;
    let variance: f32 = output_slice
        .iter()
        .map(|&x| {
            let diff = x as f32 - mean;
            diff * diff
        })
        .sum::<f32>()
        / 4.0;

    // Variance should be relatively small due to filtering
    assert!(variance < 20.0, "Variance {} too high - filtering not working", variance);
}

#[test]
fn test_no_filter_passes_through() {
    let config = Config {
        input_min: 0_u16,
        input_max: 100_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::none(),
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::None,
        snap_zones: &EMPTY_SNAP_ZONES,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // No filtering - direct mapping
    assert_eq!(pot.update(0), 0.0);
    assert_eq!(pot.update(50), 0.5);
    assert_eq!(pot.update(100), 1.0);
}

#[cfg(all(feature = "filter-ema", feature = "hysteresis-threshold"))]
#[test]
fn test_filter_combined_with_hysteresis() {
    let config = Config {
        input_min: 0_u16,
        input_max: 1000_u16,
        output_min: 0.0_f32,
        output_max: 1.0_f32,
        hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.1 },
        curve: ResponseCurve::Linear,
        filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.5 },
        snap_zones: &EMPTY_SNAP_ZONES,
    };

    let mut pot = PotHead::new(config).expect("Valid config");

    // Initialize at midpoint
    let initial = pot.update(500);
    assert!((initial - 0.5).abs() < 0.001);

    // Small change - filtered but still below hysteresis threshold
    let out2 = pot.update(550);  // Filtered to ~0.525, below 0.1 threshold
    // Should stay at 0.5 due to hysteresis
    assert_eq!(out2, initial);

    // Large change - exceeds hysteresis after filtering
    let out3 = pot.update(800);
    assert!(out3 > initial);
}
