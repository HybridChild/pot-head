use heapless::Vec;

/// Simple moving average filter state
///
/// Maintains a circular buffer of past samples. RAM cost: window_size * 4 bytes.
#[derive(Debug, Clone)]
pub struct MovingAvgFilter {
    buffer: Vec<f32, 32>, // Max window size of 32
    window_size: usize,
    index: usize,
    count: usize,
}

impl MovingAvgFilter {
    /// Create new moving average filter
    ///
    /// window_size must be > 0 and <= 32
    pub fn new(window_size: usize) -> Self {
        debug_assert!(window_size > 0 && window_size <= 32);

        let mut buffer = Vec::new();
        // Pre-fill buffer with zeros
        for _ in 0..window_size.min(32) {
            let _ = buffer.push(0.0);
        }

        Self {
            buffer,
            window_size,
            index: 0,
            count: 0,
        }
    }

    /// Apply moving average filter
    ///
    /// Averages the last window_size samples. Until buffer is full,
    /// averages all samples received so far.
    pub fn apply(&mut self, input: f32) -> f32 {
        // Store input in circular buffer
        self.buffer[self.index] = input;
        self.index = (self.index + 1) % self.window_size;

        // Track how many samples we've seen
        if self.count < self.window_size {
            self.count += 1;
        }

        // Calculate average of samples collected so far
        let sum: f32 = self.buffer.iter().take(self.count).sum();
        sum / self.count as f32
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        for val in self.buffer.iter_mut() {
            *val = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_call_returns_input() {
        let mut filter = MovingAvgFilter::new(4);
        assert_eq!(filter.apply(0.5), 0.5);
    }

    #[test]
    fn averages_samples() {
        let mut filter = MovingAvgFilter::new(3);

        filter.apply(1.0);
        filter.apply(2.0);
        let avg = filter.apply(3.0);

        // Average of [1.0, 2.0, 3.0] = 2.0
        assert!((avg - 2.0).abs() < 1e-6);
    }

    #[test]
    fn circular_buffer_wraps() {
        let mut filter = MovingAvgFilter::new(3);

        filter.apply(1.0);
        filter.apply(2.0);
        filter.apply(3.0);
        let avg = filter.apply(4.0);

        // Buffer now contains [4.0, 2.0, 3.0]
        // Average = 3.0
        assert!((avg - 3.0).abs() < 1e-6);
    }

    #[test]
    fn smooths_noise() {
        let mut filter = MovingAvgFilter::new(4);

        let samples = [1.0, 1.1, 0.9, 1.0];
        let mut outputs: Vec<f32, 4> = Vec::new();

        for &sample in &samples {
            let _ = outputs.push(filter.apply(sample));
        }

        // Last output should be average of all samples
        let expected = (1.0 + 1.1 + 0.9 + 1.0) / 4.0;
        assert!((outputs[3] - expected).abs() < 1e-6);
    }

    #[test]
    fn reset_clears_buffer() {
        let mut filter = MovingAvgFilter::new(3);

        filter.apply(5.0);
        filter.apply(5.0);
        filter.apply(5.0);

        filter.reset();

        // After reset, first sample should return itself
        assert_eq!(filter.apply(1.0), 1.0);
    }
}
