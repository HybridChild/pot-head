use crate::color_scheme::ColorScheme;
use pot_head::PotHead;

pub struct PotDisplay {
    pub pot: PotHead<u16, f32>,
    pub label: &'static str,
    pub color_scheme: ColorScheme,
    pub precision: usize,
    pub last_output: f32,
}

impl PotDisplay {
    pub fn new(
        pot: PotHead<u16, f32>,
        label: &'static str,
        color_scheme: ColorScheme,
        precision: usize,
    ) -> Self {
        // Initialize last_output to the minimum of the output range
        let config = pot.config();
        let initial_output = config.output_min.min(config.output_max);

        Self {
            pot,
            label,
            color_scheme,
            precision,
            last_output: initial_output,
        }
    }

    pub fn output_range(&self) -> (f32, f32) {
        let config = self.pot.config();
        (config.output_min, config.output_max)
    }

    pub fn update(&mut self, input: u16) -> f32 {
        self.last_output = self.pot.update(input);
        self.last_output
    }

    pub fn active_color_scheme(&self, is_selected: bool) -> ColorScheme {
        if is_selected {
            self.color_scheme
        } else {
            self.color_scheme.dimmed()
        }
    }
}
