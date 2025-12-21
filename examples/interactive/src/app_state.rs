use crate::pot_display::PotDisplay;
use crate::pot_spec::{REVERSED_POT, STANDARD_POT};
use std::io::Result;

// Input range (simulating ADC values)
const INPUT_MIN: u16 = 0;
const INPUT_MAX: u16 = 99;
const STEP_SIZE: u16 = 1;

pub struct AppState {
    pub input_value: u16,
    pub pots: Vec<PotDisplay>,
    pub selected_pot_index: usize,
    pub running: bool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let pots = vec![
            STANDARD_POT.build(INPUT_MIN, INPUT_MAX)?,
            REVERSED_POT.build(INPUT_MIN, INPUT_MAX)?,
        ];

        Ok(Self {
            input_value: INPUT_MIN,
            pots,
            selected_pot_index: 0,
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
