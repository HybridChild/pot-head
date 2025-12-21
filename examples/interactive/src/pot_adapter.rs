use crate::color_scheme::ColorScheme;
use crate::renderable_pot::{RenderInfo, RenderablePot};
use num_traits::AsPrimitive;
use pot_head::PotHead;
use std::fmt::Display;

/// Adapts a PotHead<TIn, TOut> to the RenderablePot trait
pub struct PotAdapter<TIn, TOut> {
    pot: PotHead<TIn, TOut>,
    label: &'static str,
    color_scheme: ColorScheme,
    precision: usize,
    last_output: TOut,
    // Store input range for denormalization
    input_min: TIn,
    input_max: TIn,
}

impl<TIn, TOut> PotAdapter<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32>,
    TOut: Copy + PartialOrd + AsPrimitive<f32> + Display,
    f32: AsPrimitive<TIn> + AsPrimitive<TOut>,
{
    pub fn new(
        pot: PotHead<TIn, TOut>,
        label: &'static str,
        color_scheme: ColorScheme,
        precision: usize,
        input_min: TIn,
        input_max: TIn,
    ) -> Self {
        // Initialize last_output to the minimum of the output range
        let config = pot.config();
        let initial_output = if config.output_min.as_() < config.output_max.as_() {
            config.output_min
        } else {
            config.output_max
        };

        Self {
            pot,
            label,
            color_scheme,
            precision,
            last_output: initial_output,
            input_min,
            input_max,
        }
    }

    /// Convert normalized input (0.0-1.0) to actual input type
    fn denormalize_input(&self, normalized: f32) -> TIn {
        let min_f = self.input_min.as_();
        let max_f = self.input_max.as_();
        let value_f = min_f + normalized * (max_f - min_f);
        value_f.as_()
    }

    /// Get the output range as (min, max) in ascending order
    fn output_range(&self) -> (TOut, TOut) {
        let config = self.pot.config();
        (config.output_min, config.output_max)
    }
}

impl<TIn, TOut> RenderablePot for PotAdapter<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32>,
    TOut: Copy + PartialOrd + AsPrimitive<f32> + Display,
    f32: AsPrimitive<TIn> + AsPrimitive<TOut>,
{
    fn update(&mut self, normalized_input: f32) {
        let input = self.denormalize_input(normalized_input);
        self.last_output = self.pot.update(input);
    }

    fn get_render_info(&self) -> RenderInfo {
        // Get the last input value by getting it from the pot's config and last state
        // We'll store it when we update
        let (output_min, output_max) = self.output_range();

        let input_min_f = self.input_min.as_();
        let input_max_f = self.input_max.as_();
        let output_min_f = output_min.as_();
        let output_max_f = output_max.as_();
        let output_f = self.last_output.as_();

        // Calculate the current input value from the output (reverse the mapping)
        // This works because we know: output = output_min + normalized * (output_max - output_min)
        // So: normalized = (output - output_min) / (output_max - output_min)
        let normalized = if output_max_f != output_min_f {
            (output_f - output_min_f) / (output_max_f - output_min_f)
        } else {
            0.5
        };
        let input_f = input_min_f + normalized * (input_max_f - input_min_f);

        // Determine display range (always ascending for the bar)
        let (display_min_f, display_max_f) = if output_min_f < output_max_f {
            (output_min_f, output_max_f)
        } else {
            (output_max_f, output_min_f)
        };

        // Calculate output position in the display range (0.0 = left/min, 1.0 = right/max)
        let output_position = if display_max_f != display_min_f {
            (output_f - display_min_f) / (display_max_f - display_min_f)
        } else {
            0.5
        };

        // Format output range for display
        let (display_min, display_max) = if output_min_f < output_max_f {
            (output_min, output_max)
        } else {
            (output_max, output_min)
        };

        // Determine input precision (0 for integers, same as output for floats)
        let input_precision = if input_min_f.fract() == 0.0 && input_max_f.fract() == 0.0 {
            0
        } else {
            self.precision
        };

        RenderInfo {
            label: self.label.to_string(),
            input_value: format!("{:.prec$}", input_f, prec = input_precision),
            input_range: (
                format!("{:.prec$}", input_min_f, prec = input_precision),
                format!("{:.prec$}", input_max_f, prec = input_precision),
            ),
            output_value: format!("{:.prec$}", output_f, prec = self.precision),
            output_range: (
                format!("{:.prec$}", display_min.as_(), prec = self.precision),
                format!("{:.prec$}", display_max.as_(), prec = self.precision),
            ),
            output_position,
        }
    }

    fn active_color_scheme(&self, is_selected: bool) -> ColorScheme {
        if is_selected {
            self.color_scheme
        } else {
            self.color_scheme.dimmed()
        }
    }
}
