//! Response curve implementations.
//!
//! Transforms normalized input (0.0..1.0) through different response curves.

/// Response curve types for potentiometer output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseCurve {
    /// Linear response (1:1 mapping).
    Linear,

    /// Logarithmic response (audio taper).
    ///
    /// Requires `log-curve` feature and `libm` dependency.
    #[cfg(feature = "log-curve")]
    Logarithmic,
}

impl ResponseCurve {
    /// Apply the response curve to a normalized value (0.0..1.0).
    ///
    /// Returns the transformed value, also in the range 0.0..1.0.
    #[inline]
    pub fn apply(&self, normalized: f32) -> f32 {
        match self {
            ResponseCurve::Linear => normalized,

            #[cfg(feature = "log-curve")]
            ResponseCurve::Logarithmic => apply_logarithmic(normalized),
        }
    }
}

/// Apply logarithmic (audio taper) curve.
///
/// Uses exponential function to create logarithmic response:
/// output = (e^(3x) - 1) / (e^3 - 1)
///
/// This gives perceptually linear volume control where:
/// - Lower values spread out (fine control at low volumes)
/// - Higher values compress (coarse control at high volumes)
#[cfg(feature = "log-curve")]
#[inline]
fn apply_logarithmic(normalized: f32) -> f32 {
    const E3_MINUS_1: f32 = 19.085_537; // e^3 - 1 precomputed

    // Clamp to valid range to prevent edge cases
    let x = normalized.clamp(0.0, 1.0);

    // Compute e^(3x) using libm
    let exp_3x = libm::expf(3.0 * x);

    // Apply formula: (e^(3x) - 1) / (e^3 - 1)
    (exp_3x - 1.0) / E3_MINUS_1
}
