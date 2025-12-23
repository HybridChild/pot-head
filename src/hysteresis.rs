use core::marker::PhantomData;

/// Schmitt trigger output state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SchmittState {
    Low,
    High,
}

/// Hysteresis modes for noise reduction and oscillation prevention.
/// Operates on normalized values (0.0-1.0) in the processing pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HysteresisMode<T> {
    /// No hysteresis applied
    None(PhantomData<T>),

    /// Ignore changes smaller than threshold
    ChangeThreshold { threshold: T },

    /// Separate rising/falling thresholds to prevent boundary oscillation
    SchmittTrigger { rising: T, falling: T },
}

/// State for hysteresis processing.
/// Type parameter T matches the normalized value type (typically f32).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HysteresisState<T> {
    pub last_output: T,
    pub schmitt_state: SchmittState,
}

impl<T> Default for HysteresisState<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            last_output: T::default(),
            schmitt_state: SchmittState::Low,
        }
    }
}

impl<T> HysteresisMode<T> {
    pub const fn none() -> Self {
        HysteresisMode::None(PhantomData)
    }
}

impl<T> HysteresisMode<T>
where
    T: Copy + PartialOrd + core::ops::Sub<Output = T> + core::ops::Add<Output = T>,
{
    pub fn apply(&self, input: T, state: &mut HysteresisState<T>) -> T {
        match self {
            HysteresisMode::None(_) => input,

            HysteresisMode::ChangeThreshold { threshold } => {
                // Calculate absolute difference between input and last output
                let diff = if input > state.last_output {
                    input - state.last_output
                } else {
                    state.last_output - input
                };

                // Only update if change exceeds threshold
                let output = if diff > *threshold {
                    input
                } else {
                    state.last_output
                };

                state.last_output = output;
                output
            }

            HysteresisMode::SchmittTrigger { rising, falling } => {
                // Update state based on thresholds
                if input >= *rising {
                    state.schmitt_state = SchmittState::High;
                } else if input <= *falling {
                    state.schmitt_state = SchmittState::Low;
                }

                // Output depends on current state
                let output = match state.schmitt_state {
                    SchmittState::High => *rising,
                    SchmittState::Low => *falling,
                };

                state.last_output = output;
                output
            }
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            HysteresisMode::None(_) => Ok(()),

            HysteresisMode::ChangeThreshold { .. } => Ok(()),

            HysteresisMode::SchmittTrigger { rising, falling } => {
                if rising <= falling {
                    Err("Schmitt trigger: rising threshold must be greater than falling threshold")
                } else {
                    Ok(())
                }
            }
        }
    }
}
