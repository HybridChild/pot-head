use crate::color_scheme::ColorScheme;
use crate::pot_display::PotDisplay;
use crossterm::style::Color;
use pot_head::{Config, PotHead};
use std::io::Result;

// Input range (simulating ADC values)
const INPUT_MIN: u16 = 0;
const INPUT_MAX: u16 = 99;
const STEP_SIZE: u16 = 1;

// Standard potmeter output range
const OUTPUT_MIN: f32 = 0.0;
const OUTPUT_MAX: f32 = 1.0;

// Reversed polarity potmeter
const REVERSED_MIN: f32 = 100.0;
const REVERSED_MAX: f32 = -100.0;

pub struct AppState {
    pub input_value: u16,
    pub pots: Vec<PotDisplay>,
    pub selected_pot_index: usize,
    pub running: bool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let mut pots = Vec::new();

        // Standard potmeter
        let config_standard = Config {
            input_min: INPUT_MIN,
            input_max: INPUT_MAX,
            output_min: OUTPUT_MIN,
            output_max: OUTPUT_MAX,
        };

        let pot_standard = PotHead::new(config_standard).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("PotHead config error: {:?}", e),
            )
        })?;

        pots.push(PotDisplay::new(
            pot_standard,
            "Standard Pot",
            ColorScheme::new(
                Color::Rgb { r: 0, g: 255, b: 0 },
                Color::Rgb {
                    r: 0,
                    g: 200,
                    b: 255,
                }, // Cyan
                Color::Rgb {
                    r: 255,
                    g: 165,
                    b: 0,
                }, // Orange
            ),
            4,
        ));

        // Reversed polarity potmeter
        let config_reversed = Config {
            input_min: INPUT_MIN,
            input_max: INPUT_MAX,
            output_min: REVERSED_MIN,
            output_max: REVERSED_MAX,
        };

        let pot_reversed = PotHead::new(config_reversed).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("PotHead reversed config error: {:?}", e),
            )
        })?;

        pots.push(PotDisplay::new(
            pot_reversed,
            "Reversed Pot",
            ColorScheme::new(
                Color::Rgb {
                    r: 255,
                    g: 0,
                    b: 255,
                },
                Color::Rgb {
                    r: 255,
                    g: 100,
                    b: 100,
                }, // Pink
                Color::Rgb {
                    r: 255,
                    g: 200,
                    b: 0,
                }, // Yellow
            ),
            2,
        ));

        Ok(Self {
            input_value: INPUT_MIN,
            pots,
            selected_pot_index: 0, // Start with first pot selected
            running: true,
        })
    }

    pub fn increase_input(&mut self) {
        self.input_value = self.input_value.saturating_add(STEP_SIZE).min(INPUT_MAX);
    }

    pub fn decrease_input(&mut self) {
        self.input_value = self.input_value.saturating_sub(STEP_SIZE).max(INPUT_MIN);
    }

    pub fn select_next_pot(&mut self) {
        if !self.pots.is_empty() {
            self.selected_pot_index = (self.selected_pot_index + 1) % self.pots.len();
        }
    }

    pub fn select_prev_pot(&mut self) {
        if !self.pots.is_empty() {
            self.selected_pot_index = if self.selected_pot_index == 0 {
                self.pots.len() - 1
            } else {
                self.selected_pot_index - 1
            };
        }
    }

    pub fn input_min() -> u16 {
        INPUT_MIN
    }

    pub fn input_max() -> u16 {
        INPUT_MAX
    }
}
