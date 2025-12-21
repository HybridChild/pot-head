use crate::color_scheme::ColorScheme;

/// Information needed to render a pot in the UI
pub struct RenderInfo {
    pub label: String,
    pub input_value: String,           // Formatted input value
    pub input_range: (String, String), // (min, max) formatted for display
    pub output_value: String,          // Formatted output value
    pub output_range: (String, String), // (min, max) formatted for display
    pub output_position: f32,          // Normalized 0.0-1.0 for bar rendering
}

/// Trait for pots that can be rendered in the interactive demo.
/// This abstracts over specific input/output types.
pub trait RenderablePot {
    /// Update the pot with a normalized input value (0.0 = min, 1.0 = max)
    fn update(&mut self, normalized_input: f32);

    /// Get rendering information for this pot
    fn get_render_info(&self) -> RenderInfo;

    /// Get the color scheme, accounting for selection state
    fn active_color_scheme(&self, is_selected: bool) -> ColorScheme;
}
