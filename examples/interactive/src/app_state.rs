use crate::pot_spec::{INTEGER_POT, REVERSED_POT, STANDARD_POT};
use crate::renderable_pot::RenderablePot;
use crate::rendering::BAR_WIDTH;
use rand_distr::{Distribution, Normal};
use std::io::Result;

// Step size matches bar width so each arrow key press moves one position on the bar
const STEP_SIZE: f32 = 1.0 / BAR_WIDTH as f32;
const NOISE_STEP_SIZE: f32 = 0.05; // 5% increments for noise level

pub struct AppState {
    pub normalized_input: f32, // Always 0.0 to 1.0 (clean input before noise)
    pub noise_level: f32,       // 0.0 to 1.0, controls noise amplitude
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
            noise_level: 0.05,               // Start with 5% noise
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

    pub fn increase_noise(&mut self) {
        self.noise_level = (self.noise_level + NOISE_STEP_SIZE).min(1.0);
    }

    pub fn decrease_noise(&mut self) {
        self.noise_level = (self.noise_level - NOISE_STEP_SIZE).max(0.0);
    }

    /// Get the current input with noise applied
    /// Uses Gaussian noise scaled by noise_level
    pub fn get_noisy_input(&self) -> f32 {
        if self.noise_level == 0.0 {
            return self.normalized_input;
        }

        let mut rng = rand::thread_rng();

        // Standard deviation scales with noise_level
        // Max noise is ~10% of full range (3 sigma rule: 99.7% within ±3σ)
        let sigma = self.noise_level * 0.033;

        // Create normal distribution centered at 0
        let normal = Normal::new(0.0, sigma as f64).unwrap();
        let noise = normal.sample(&mut rng) as f32;

        // Apply noise and clamp to valid range
        (self.normalized_input + noise).clamp(0.0, 1.0)
    }
}
