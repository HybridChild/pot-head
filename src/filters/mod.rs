/// Noise filtering implementations
///
/// Filters smooth noisy ADC readings. All filtering happens in normalized f32 space.

#[cfg(feature = "filter-ema")]
mod ema;
#[cfg(feature = "filter-moving-avg")]
mod moving_avg;

#[cfg(feature = "filter-ema")]
pub use ema::EmaFilter;
#[cfg(feature = "filter-moving-avg")]
pub use moving_avg::MovingAvgFilter;

/// Noise filter configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoiseFilter {
    /// No filtering applied
    None,

    /// Exponential moving average: output = alpha * input + (1 - alpha) * previous
    /// Lower alpha = more smoothing, higher = more responsive
    /// Requires: 0.0 < alpha <= 1.0
    #[cfg(feature = "filter-ema")]
    ExponentialMovingAverage { alpha: f32 },

    /// Simple moving average over N samples
    /// Window size configured at filter creation
    /// Requires buffer of size window_size (RAM cost: window_size * 4 bytes)
    #[cfg(feature = "filter-moving-avg")]
    MovingAverage { window_size: usize },
}

impl NoiseFilter {
    /// Validate filter configuration at compile time
    pub const fn validate(&self) -> Result<(), &'static str> {
        match self {
            NoiseFilter::None => Ok(()),

            #[cfg(feature = "filter-ema")]
            NoiseFilter::ExponentialMovingAverage { alpha } => {
                if *alpha <= 0.0 || *alpha > 1.0 {
                    return Err("EMA alpha must be in range (0.0, 1.0]");
                }
                Ok(())
            }

            #[cfg(feature = "filter-moving-avg")]
            NoiseFilter::MovingAverage { window_size } => {
                if *window_size == 0 {
                    return Err("MovingAverage window_size must be > 0");
                }
                if *window_size > 32 {
                    return Err("MovingAverage window_size must be <= 32");
                }
                Ok(())
            }
        }
    }
}
