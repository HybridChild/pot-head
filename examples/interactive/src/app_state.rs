use crate::pot_spec::{INTEGER_POT, REVERSED_POT, STANDARD_POT};
use crate::renderable_pot::RenderablePot;
use std::io::Result;

// Normalized input range (always 0.0 to 1.0)
const STEP_SIZE: f32 = 0.01; // 1% steps

pub struct AppState {
    pub normalized_input: f32, // Always 0.0 to 1.0
    pub pots: Vec<Box<dyn RenderablePot>>,
    pub selected_pot_index: usize,
    pub running: bool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let pots: Vec<Box<dyn RenderablePot>> = vec![
            STANDARD_POT.build()?,
            REVERSED_POT.build()?,
            INTEGER_POT.build()?,
        ];

        Ok(Self {
            normalized_input: 0.0,
            pots,
            selected_pot_index: 0,
            running: true,
        })
    }

    pub fn increase_input(&mut self) {
        self.normalized_input = (self.normalized_input + STEP_SIZE).min(1.0);
    }

    pub fn decrease_input(&mut self) {
        self.normalized_input = (self.normalized_input - STEP_SIZE).max(0.0);
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
}
