use num_traits::AsPrimitive;

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    InvalidInputRange,
    InvalidOutputRange,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidInputRange => write!(f, "input_min must be less than input_max"),
            ConfigError::InvalidOutputRange => write!(f, "output_min must not equal output_max"),
        }
    }
}

pub struct Config<TIn, TOut = TIn> {
    pub input_min: TIn,
    pub input_max: TIn,
    pub output_min: TOut,
    pub output_max: TOut,
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

        Ok(())
    }
}
