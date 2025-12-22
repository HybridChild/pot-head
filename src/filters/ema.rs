/// Exponential Moving Average filter state
#[derive(Debug, Clone, Copy)]
pub struct EmaFilter {
    previous: f32,
    initialized: bool,
}

impl EmaFilter {
    /// Create new EMA filter with uninitialized state
    pub const fn new() -> Self {
        Self {
            previous: 0.0,
            initialized: false,
        }
    }

    /// Apply EMA filter: output = alpha * input + (1 - alpha) * previous
    ///
    /// First call initializes the filter to the input value.
    pub fn apply(&mut self, input: f32, alpha: f32) -> f32 {
        debug_assert!(
            alpha > 0.0 && alpha <= 1.0,
            "EMA alpha must be in range (0.0, 1.0], got {}",
            alpha
        );

        if !self.initialized {
            self.previous = input;
            self.initialized = true;
            return input;
        }

        let output = alpha * input + (1.0 - alpha) * self.previous;
        self.previous = output;
        output
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.initialized = false;
        self.previous = 0.0;
    }
}

impl Default for EmaFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_call_returns_input() {
        let mut filter = EmaFilter::new();
        assert_eq!(filter.apply(0.5, 0.3), 0.5);
    }

    #[test]
    fn applies_smoothing() {
        let mut filter = EmaFilter::new();
        filter.apply(0.0, 0.3);

        // Step input from 0.0 to 1.0
        let output = filter.apply(1.0, 0.3);
        // output = 0.3 * 1.0 + 0.7 * 0.0 = 0.3
        assert!((output - 0.3).abs() < 1e-6);
    }

    #[test]
    fn lower_alpha_more_smoothing() {
        let mut filter_low = EmaFilter::new();
        let mut filter_high = EmaFilter::new();

        filter_low.apply(0.0, 0.1);
        filter_high.apply(0.0, 0.9);

        let out_low = filter_low.apply(1.0, 0.1);
        let out_high = filter_high.apply(1.0, 0.9);

        // Higher alpha should respond more to new input
        assert!(out_high > out_low);
    }

    #[test]
    fn reset_reinitializes() {
        let mut filter = EmaFilter::new();
        filter.apply(0.5, 0.3);
        filter.apply(0.7, 0.3);

        filter.reset();

        // After reset, first call should return input directly
        assert_eq!(filter.apply(1.0, 0.3), 1.0);
    }
}
