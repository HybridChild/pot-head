use num_traits::AsPrimitive;

use crate::config::{Config, ConfigError};
use crate::state::State;

pub struct PotHead<TIn, TOut = TIn> {
    config: Config<TIn, TOut>,
    state: State<f32>,
}

impl<TIn, TOut> PotHead<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32>,
    TOut: Copy + PartialOrd + AsPrimitive<f32>,
    f32: AsPrimitive<TOut>,
{
    pub fn new(config: Config<TIn, TOut>) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Self {
            config,
            state: State::default(),
        })
    }

    pub fn config(&self) -> &Config<TIn, TOut> {
        &self.config
    }

    pub fn update(&mut self, input: TIn) -> TOut {
        // Normalize input to 0.0..1.0
        let normalized = self.normalize_input(input);

        // Apply hysteresis
        let processed = self.config.hysteresis.apply(normalized, &mut self.state.hysteresis);

        // Denormalize to output range
        self.denormalize_output(processed)
    }

    fn normalize_input(&self, input: TIn) -> f32 {
        let input_f = input.as_();
        let min_f = self.config.input_min.as_();
        let max_f = self.config.input_max.as_();

        // Clamp input to valid range
        let clamped = if input_f < min_f {
            min_f
        } else if input_f > max_f {
            max_f
        } else {
            input_f
        };

        // Normalize to 0.0..1.0
        // Safe division: validation ensures max_f > min_f
        (clamped - min_f) / (max_f - min_f)
    }

    fn denormalize_output(&self, normalized: f32) -> TOut {
        let min_f = self.config.output_min.as_();
        let max_f = self.config.output_max.as_();

        let output_f = min_f + normalized * (max_f - min_f);
        output_f.as_()
    }
}
