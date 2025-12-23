use num_traits::AsPrimitive;

use crate::config::{Config, ConfigError};
use crate::state::State;
use crate::filters::NoiseFilter;

#[cfg(feature = "filter-ema")]
use crate::filters::EmaFilter;
#[cfg(feature = "filter-moving-avg")]
use crate::filters::MovingAvgFilter;

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

        let mut state = State::default();

        // Initialize filter state based on configuration
        #[cfg(feature = "filter-ema")]
        if matches!(config.filter, NoiseFilter::ExponentialMovingAverage { .. }) {
            state.ema_filter = Some(EmaFilter::new());
        }

        #[cfg(feature = "filter-moving-avg")]
        if let NoiseFilter::MovingAverage { window_size } = config.filter {
            state.ma_filter = Some(MovingAvgFilter::new(window_size));
        }

        Ok(Self { config, state })
    }

    pub fn config(&self) -> &Config<TIn, TOut> {
        &self.config
    }

    pub fn update(&mut self, input: TIn) -> TOut {
        // Normalize input to 0.0..1.0
        let normalized = self.normalize_input(input);

        // Apply noise filter
        let filtered = self.apply_filter(normalized);

        // Apply response curve
        let curved = self.config.curve.apply(filtered);

        // Apply hysteresis
        let hysteresis_applied = self.config.hysteresis.apply(curved, &mut self.state.hysteresis);

        // Apply snap zones
        let snapped = self.apply_snap_zones(hysteresis_applied);

        // Update last output for dead zones
        self.state.last_output = snapped;

        // Denormalize to output range
        self.denormalize_output(snapped)
    }

    fn apply_filter(&mut self, value: f32) -> f32 {
        match &self.config.filter {
            NoiseFilter::None => value,

            #[cfg(feature = "filter-ema")]
            NoiseFilter::ExponentialMovingAverage { alpha } => {
                if let Some(ref mut filter) = self.state.ema_filter {
                    filter.apply(value, *alpha)
                } else {
                    value
                }
            }

            #[cfg(feature = "filter-moving-avg")]
            NoiseFilter::MovingAverage { .. } => {
                if let Some(ref mut filter) = self.state.ma_filter {
                    filter.apply(value)
                } else {
                    value
                }
            }
        }
    }

    fn apply_snap_zones(&self, value: f32) -> f32 {
        // Process zones in order - first match wins
        for zone in self.config.snap_zones {
            if zone.contains(value) {
                return zone.apply(value, self.state.last_output);
            }
        }
        value // No zone matched
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
