#![cfg(any(feature = "filter-ema", feature = "filter-moving-avg"))]

#[cfg(feature = "filter-ema")]
mod ema_tests {
    use pot_head::filters::EmaFilter;

    #[test]
    fn ema_initialization() {
        let mut filter = EmaFilter::new();
        // First value initializes the filter
        let out = filter.apply(0.8, 0.5);
        assert_eq!(out, 0.8);
    }

    #[test]
    fn ema_step_response() {
        let mut filter = EmaFilter::new();
        filter.apply(0.0, 0.5);

        // Step from 0.0 to 1.0
        let out1 = filter.apply(1.0, 0.5);
        assert!((out1 - 0.5).abs() < 1e-6, "Expected 0.5, got {}", out1);

        let out2 = filter.apply(1.0, 0.5);
        assert!((out2 - 0.75).abs() < 1e-6, "Expected 0.75, got {}", out2);
    }

    #[test]
    fn ema_converges_to_constant() {
        let mut filter = EmaFilter::new();
        filter.apply(0.0, 0.3);

        let mut output = 0.0;
        for _ in 0..100 {
            output = filter.apply(1.0, 0.3);
        }

        // After many iterations, should converge to 1.0
        assert!((output - 1.0).abs() < 0.01);
    }

    #[test]
    fn ema_alpha_1_no_filtering() {
        let mut filter = EmaFilter::new();
        filter.apply(0.0, 1.0);

        // Alpha = 1.0 means no filtering (immediate response)
        assert_eq!(filter.apply(0.5, 1.0), 0.5);
        assert_eq!(filter.apply(0.9, 1.0), 0.9);
    }

    #[test]
    fn ema_filters_noise() {
        let mut filter = EmaFilter::new();

        // Noisy signal around 0.5
        let noisy_samples = [0.5, 0.6, 0.4, 0.55, 0.45, 0.52];
        let mut outputs = Vec::new();

        for &sample in &noisy_samples {
            outputs.push(filter.apply(sample, 0.2));
        }

        // Check that variance of output is less than variance of input
        let input_variance = variance(&noisy_samples);
        let output_variance = variance(&outputs);

        assert!(output_variance < input_variance);
    }

    fn variance(data: &[f32]) -> f32 {
        let mean: f32 = data.iter().sum::<f32>() / data.len() as f32;
        let variance: f32 = data.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / data.len() as f32;
        variance
    }
}

#[cfg(feature = "filter-moving-avg")]
mod moving_avg_tests {
    use pot_head::filters::MovingAvgFilter;

    #[test]
    fn moving_avg_initialization() {
        let mut filter = MovingAvgFilter::new(5);
        let out = filter.apply(0.7);
        assert_eq!(out, 0.7);
    }

    #[test]
    fn moving_avg_fills_buffer() {
        let mut filter = MovingAvgFilter::new(4);

        assert_eq!(filter.apply(1.0), 1.0);
        assert_eq!(filter.apply(2.0), 1.5);
        assert_eq!(filter.apply(3.0), 2.0);
        assert_eq!(filter.apply(4.0), 2.5);
    }

    #[test]
    fn moving_avg_window_slides() {
        let mut filter = MovingAvgFilter::new(3);

        filter.apply(1.0);
        filter.apply(2.0);
        filter.apply(3.0);

        // Buffer: [1.0, 2.0, 3.0], avg = 2.0
        let out1 = filter.apply(6.0);
        // Buffer: [6.0, 2.0, 3.0], avg = 3.666...
        assert!((out1 - 3.666666).abs() < 0.001);

        let out2 = filter.apply(9.0);
        // Buffer: [6.0, 9.0, 3.0], avg = 6.0
        assert!((out2 - 6.0).abs() < 0.001);
    }

    #[test]
    fn moving_avg_constant_input() {
        let mut filter = MovingAvgFilter::new(5);

        for _ in 0..10 {
            let out = filter.apply(0.42);
            assert!((out - 0.42).abs() < 1e-6);
        }
    }

    #[test]
    fn moving_avg_filters_spike() {
        let mut filter = MovingAvgFilter::new(5);

        // Establish baseline
        filter.apply(1.0);
        filter.apply(1.0);
        filter.apply(1.0);
        filter.apply(1.0);

        // Single spike
        let out = filter.apply(5.0);
        // Buffer: [1.0, 1.0, 1.0, 1.0, 5.0], avg = 1.8
        assert!((out - 1.8).abs() < 0.001);
    }

    #[test]
    fn moving_avg_step_response() {
        let mut filter = MovingAvgFilter::new(4);

        // Step from 0.0 to 1.0
        filter.apply(0.0);
        filter.apply(0.0);
        filter.apply(0.0);
        filter.apply(0.0);

        let out1 = filter.apply(1.0);
        assert!((out1 - 0.25).abs() < 0.001);

        let out2 = filter.apply(1.0);
        assert!((out2 - 0.5).abs() < 0.001);

        let out3 = filter.apply(1.0);
        assert!((out3 - 0.75).abs() < 0.001);

        let out4 = filter.apply(1.0);
        assert!((out4 - 1.0).abs() < 0.001);
    }
}

#[cfg(all(feature = "filter-ema", feature = "filter-moving-avg"))]
mod comparison_tests {
    use pot_head::filters::{EmaFilter, MovingAvgFilter};

    #[test]
    fn both_smooth_noise() {
        let mut ema = EmaFilter::new();
        let mut ma = MovingAvgFilter::new(5);

        let noisy = [1.0, 1.1, 0.9, 1.05, 0.95, 1.0];

        let ema_outputs: Vec<f32> = noisy.iter()
            .map(|&x| ema.apply(x, 0.3))
            .collect();

        let ma_outputs: Vec<f32> = noisy.iter()
            .map(|&x| ma.apply(x))
            .collect();

        // Both should produce smoother output than input
        // Just verify they produce reasonable values
        assert!(ema_outputs.iter().all(|&x| x >= 0.9 && x <= 1.1));
        assert!(ma_outputs.iter().all(|&x| x >= 0.9 && x <= 1.1));
    }

    #[test]
    fn ema_responds_faster_than_ma() {
        let mut ema = EmaFilter::new();
        let mut ma = MovingAvgFilter::new(8);

        // Initialize both to 0.0
        ema.apply(0.0, 0.5);
        for _ in 0..8 {
            ma.apply(0.0);
        }

        // Step to 1.0
        let ema_out = ema.apply(1.0, 0.5);
        let ma_out = ma.apply(1.0);

        // EMA with alpha=0.5 should respond faster than MA with window=8
        assert!(ema_out > ma_out);
    }
}
