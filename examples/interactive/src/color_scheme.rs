use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct ColorScheme {
    pub bar_color: Color,
    pub processed_indicator_color: Color,
    pub physical_indicator_color: Color,
    pub threshold_color: Color,
}

impl ColorScheme {
    pub fn dimmed(&self) -> ColorScheme {
        ColorScheme {
            bar_color: dim_color(self.bar_color),
            processed_indicator_color: self.processed_indicator_color,
            physical_indicator_color: self.physical_indicator_color,
            threshold_color: dim_color(self.threshold_color),
        }
    }
}

fn dim_color(color: Color) -> Color {
    match color {
        Color::Rgb { r, g, b } => Color::Rgb {
            r: r / 3,
            g: g / 3,
            b: b / 3,
        },
        other => other,
    }
}
