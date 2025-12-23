use num_traits::AsPrimitive;

use crate::curves::ResponseCurve;
use crate::filters::NoiseFilter;
use crate::hysteresis::HysteresisMode;
use crate::snap_zones::SnapZone;

#[cfg(feature = "grab-mode")]
use crate::grab_mode::GrabMode;

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    InvalidInputRange,
    InvalidOutputRange,
    InvalidHysteresis,
    InvalidFilter,
    OverlappingSnapZones,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidInputRange => write!(f, "input_min must be less than input_max"),
            ConfigError::InvalidOutputRange => write!(f, "output_min must not equal output_max"),
            ConfigError::InvalidHysteresis => write!(f, "invalid hysteresis configuration"),
            ConfigError::InvalidFilter => write!(f, "invalid filter configuration"),
            ConfigError::OverlappingSnapZones => write!(f, "snap zones must not overlap"),
        }
    }
}

pub struct Config<TIn, TOut = TIn> {
    pub input_min: TIn,
    pub input_max: TIn,
    pub output_min: TOut,
    pub output_max: TOut,
    pub hysteresis: HysteresisMode<f32>,
    pub curve: ResponseCurve,
    pub filter: NoiseFilter,
    pub snap_zones: &'static [SnapZone<f32>],

    #[cfg(feature = "grab-mode")]
    pub grab_mode: GrabMode,
}

impl<TIn, TOut> Config<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32>,
    TOut: Copy + PartialOrd + AsPrimitive<f32>,
{
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Input range must be valid (min < max)
        if self.input_min >= self.input_max {
            return Err(ConfigError::InvalidInputRange);
        }

        // Output range must not be degenerate (min == max would cause division issues)
        if self.output_min == self.output_max {
            return Err(ConfigError::InvalidOutputRange);
        }

        // Validate hysteresis configuration
        self.hysteresis
            .validate()
            .map_err(|_| ConfigError::InvalidHysteresis)?;

        // Validate filter configuration
        self.filter
            .validate()
            .map_err(|_| ConfigError::InvalidFilter)?;

        Ok(())
    }

    /// Validate that no snap zones overlap.
    /// This is an optional validation helper - overlaps are allowed by default.
    /// Call this during development if you want to ensure clean, non-overlapping zones.
    pub fn validate_snap_zones(&self) -> Result<(), ConfigError> {
        for (i, zone1) in self.snap_zones.iter().enumerate() {
            for zone2 in self.snap_zones.iter().skip(i + 1) {
                if zone1.overlaps(zone2) {
                    return Err(ConfigError::OverlappingSnapZones);
                }
            }
        }
        Ok(())
    }
}
