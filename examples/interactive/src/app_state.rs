use crate::pot_spec::{INTEGER_POT, REVERSED_POT, STANDARD_POT};
use crate::renderable_pot::RenderablePot;
use crate::rendering::BAR_WIDTH;
use std::io::Result;

// Step size matches bar width so each arrow key press moves one position on the bar
const STEP_SIZE: f32 = 1.0 / BAR_WIDTH as f32;

pub struct AppState {
    pub normalized_input: f32, // Always 0.0 to 1.0
    pub pots: Vec<Box<dyn RenderablePot>>,
    pub selected_pot_index: usize,
    pub running: bool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let mut pots: Vec<Box<dyn RenderablePot>> = vec![
            STANDARD_POT.build()?,
            REVERSED_POT.build()?,
            INTEGER_POT.build()?,
        ];

        // Initialize all pots with centered input
        let initial_input = 0.5;
        for pot in pots.iter_mut() {
            pot.update(initial_input);
        }

        Ok(Self {
            normalized_input: initial_input, // Start centered
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
