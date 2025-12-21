use crate::color_scheme::ColorScheme;
use crate::pot_adapter::PotAdapter;
use crate::renderable_pot::RenderablePot;
use crossterm::style::Color;
use num_traits::AsPrimitive;
use pot_head::{Config, PotHead};
use std::fmt::Display;
use std::io::Result;

/// Specification for creating a pot with all its display properties
pub struct PotSpec<TIn, TOut> {
    pub label: &'static str,
    pub input_min: TIn,
    pub input_max: TIn,
    pub output_min: TOut,
    pub output_max: TOut,
    pub color_scheme: ColorScheme,
    pub precision: usize,
}

impl<TIn, TOut> PotSpec<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32> + 'static,
    TOut: Copy + PartialOrd + AsPrimitive<f32> + Display + 'static,
    f32: AsPrimitive<TIn> + AsPrimitive<TOut>,
{
    pub fn build(&self) -> Result<Box<dyn RenderablePot>> {
        let config = Config {
            input_min: self.input_min,
            input_max: self.input_max,
            output_min: self.output_min,
            output_max: self.output_max,
        };

        let pot = PotHead::new(config).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} config error: {:?}", self.label, e),
            )
        })?;

        Ok(Box::new(PotAdapter::new(
            pot,
            self.label,
            self.color_scheme,
            self.precision,
            self.input_min,
            self.input_max,
        )))
    }
}

// Pre-defined pot specifications
pub const STANDARD_POT: PotSpec<u16, f32> = PotSpec {
    label: "Standard Pot",
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    color_scheme: ColorScheme {
        bar_color: Color::Rgb { r: 0, g: 255, b: 0 },
        processed_indicator_color: Color::Rgb { r: 0, g: 200, b: 255 },
        physical_indicator_color: Color::Rgb { r: 255, g: 165, b: 0 },
    },
    precision: 3,
};

pub const REVERSED_POT: PotSpec<u16, f32> = PotSpec {
    label: "Reversed Pot",
    input_min: 0,
    input_max: 4095,
    output_min: 100.0,
    output_max: -100.0,
    color_scheme: ColorScheme {
        bar_color: Color::Rgb { r: 255, g: 0, b: 255 },
        processed_indicator_color: Color::Rgb { r: 255, g: 100, b: 100 },
        physical_indicator_color: Color::Rgb { r: 255, g: 200, b: 0 },
    },
    precision: 2,
};

pub const INTEGER_POT: PotSpec<u16, i32> = PotSpec {
    label: "Integer Pot (MIDI)",
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 127,
    color_scheme: ColorScheme {
        bar_color: Color::Rgb { r: 100, g: 150, b: 255 },
        processed_indicator_color: Color::Rgb { r: 50, g: 100, b: 200 },
        physical_indicator_color: Color::Rgb { r: 255, g: 165, b: 0 },
    },
    precision: 0,
};
