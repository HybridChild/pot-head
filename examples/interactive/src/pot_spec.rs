use crate::color_scheme::ColorScheme;
use crate::pot_display::PotDisplay;
use crossterm::style::Color;
use pot_head::{Config, PotHead};
use std::io::Result;

/// Specification for creating a pot with all its display properties
pub struct PotSpec {
    pub label: &'static str,
    pub output_min: f32,
    pub output_max: f32,
    pub color_scheme: ColorScheme,
    pub precision: usize,
}

impl PotSpec {
    pub fn build(&self, input_min: u16, input_max: u16) -> Result<PotDisplay> {
        let config = Config {
            input_min,
            input_max,
            output_min: self.output_min,
            output_max: self.output_max,
        };

        let pot = PotHead::new(config).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} config error: {:?}", self.label, e),
            )
        })?;

        Ok(PotDisplay::new(
            pot,
            self.label,
            self.color_scheme,
            self.precision,
        ))
    }
}

// Pre-defined pot specifications
pub const STANDARD_POT: PotSpec = PotSpec {
    label: "Standard Pot",
    output_min: 0.0,
    output_max: 1.0,
    color_scheme: ColorScheme {
        bar_color: Color::Rgb { r: 0, g: 255, b: 0 },
        processed_indicator_color: Color::Rgb { r: 0, g: 200, b: 255 },
        physical_indicator_color: Color::Rgb { r: 255, g: 165, b: 0 },
    },
    precision: 4,
};

pub const REVERSED_POT: PotSpec = PotSpec {
    label: "Reversed Pot",
    output_min: 100.0,
    output_max: -100.0,
    color_scheme: ColorScheme {
        bar_color: Color::Rgb { r: 255, g: 0, b: 255 },
        processed_indicator_color: Color::Rgb { r: 255, g: 100, b: 100 },
        physical_indicator_color: Color::Rgb { r: 255, g: 200, b: 0 },
    },
    precision: 2,
};
