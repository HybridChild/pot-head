use crate::app_state::AppState;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Result, Write};

// Bar properties
const BAR_WIDTH: usize = 100;

pub fn render_bar(
    processed_value: f32,
    physical_value: f32,
    min: f32,
    max: f32,
    width: usize,
    bar_color: Color,
    processed_indicator_color: Color,
    physical_indicator_color: Color,
) -> String {
    let processed_normalized = ((processed_value - min) / (max - min)).clamp(0.0, 1.0);
    let physical_normalized = ((physical_value - min) / (max - min)).clamp(0.0, 1.0);

    // Total character positions inside the bar (excluding the pipe characters)
    let inner_width = width - 2;

    // Calculate positions for both indicators
    let processed_pos = (processed_normalized * (inner_width - 1) as f32).round() as usize;
    let physical_pos = (physical_normalized * (inner_width - 1) as f32).round() as usize;

    let mut bar = String::with_capacity(width + 200); // Extra space for ANSI codes

    // Helper to set color
    let set_color = |color: Color| -> String {
        match color {
            Color::Rgb { r, g, b } => format!("\x1b[38;2;{};{};{}m", r, g, b),
            _ => "\x1b[38;2;255;255;255m".to_string(),
        }
    };

    // Start bar
    bar.push_str(&set_color(bar_color));
    bar.push('|');

    // Build the bar character by character
    for i in 0..inner_width {
        // Check if we need to render the processed indicator "< >" at this position
        let is_processed_left = i + 1 == processed_pos;
        let is_processed_center = i == processed_pos;
        let is_processed_right = i == processed_pos + 1;

        // Check if we need to render the physical indicator "|" at this position
        let is_physical = i == physical_pos;

        // Physical indicator has priority (drawn on top)
        if is_physical {
            bar.push_str(&set_color(physical_indicator_color));
            bar.push('|');
            bar.push_str(&set_color(bar_color));
        } else if is_processed_left {
            bar.push_str(&set_color(processed_indicator_color));
            bar.push('<');
            bar.push_str(&set_color(bar_color));
        } else if is_processed_center {
            bar.push_str(&set_color(processed_indicator_color));
            bar.push(' ');
            bar.push_str(&set_color(bar_color));
        } else if is_processed_right {
            bar.push_str(&set_color(processed_indicator_color));
            bar.push('>');
            bar.push_str(&set_color(bar_color));
        } else {
            bar.push('-');
        }
    }

    bar.push('|');

    // Reset color
    bar.push_str("\x1b[0m");

    bar
}

pub fn render(state: &mut AppState) -> Result<()> {
    let mut stdout = stdout();

    // Start with header
    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(""),
        MoveTo(0, 1),
        SetForegroundColor(Color::Blue),
        Print(
            "╔════════════════════════════════════════════════════════════════════════════════════════════════════════════╗"
        ),
        MoveTo(0, 2),
        Print(
            "║                                        pot-head Interactive Demo                                           ║"
        ),
        MoveTo(0, 3),
        Print(
            "╠════════════════════════════════════════════════════════════════════════════════════════════════════════════╣"
        ),
        ResetColor,
        MoveTo(0, 4),
        Print(""),
    )?;

    let mut line = 5;

    // Render input (normalized 0.0 - 1.0)
    queue!(
        stdout,
        MoveTo(0, line),
        SetForegroundColor(Color::Rgb {
            r: 255,
            g: 255,
            b: 0
        }),
        Print(format!(
            "     Input [0.0 - 1.0]: Current value: {:.2}",
            state.normalized_input
        )),
        ResetColor,
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print(format!(
            "     {}",
            render_bar(
                state.normalized_input,
                state.normalized_input, // Physical and processed are the same for raw input
                0.0,
                1.0,
                BAR_WIDTH,
                Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 0
                },
                Color::Rgb {
                    r: 200,
                    g: 200,
                    b: 0
                }, // Processed indicator (darker yellow)
                Color::Rgb {
                    r: 255,
                    g: 165,
                    b: 0
                } // Physical indicator (orange)
            )
        )),
    )?;
    line += 1;

    // Render each pot
    for (index, pot) in state.pots.iter_mut().enumerate() {
        let is_selected = index == state.selected_pot_index;

        // Only update the selected pot
        if is_selected {
            pot.update(state.normalized_input);
        }

        let info = pot.get_render_info();
        let colors = pot.active_color_scheme(is_selected);

        queue!(stdout, MoveTo(0, line), Print(""),)?;
        line += 1;

        // Add selection indicator to the label
        let selection_marker = if is_selected { "► " } else { "  " };

        queue!(
            stdout,
            MoveTo(0, line),
            SetForegroundColor(colors.bar_color),
            Print(format!(
                "   {} {} [{} - {}]: Current value: {}",
                selection_marker,
                info.label,
                info.output_range.0,
                info.output_range.1,
                info.output_value,
            )),
            ResetColor,
        )?;
        line += 1;

        // Physical position is the normalized input mapped to the display range
        let physical_position = state.normalized_input;

        queue!(
            stdout,
            MoveTo(0, line),
            Print(format!(
                "     {}",
                render_bar(
                    info.output_position, // Processed position (normalized output)
                    physical_position,    // Physical position (normalized input)
                    0.0,
                    1.0,
                    BAR_WIDTH,
                    colors.bar_color,
                    colors.processed_indicator_color,
                    colors.physical_indicator_color
                )
            )),
        )?;
        line += 1;
    }

    // Footer
    queue!(stdout, MoveTo(0, line), Print(""),)?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        SetForegroundColor(Color::Blue),
        Print(
            "╠════════════════════════════════════════════════════════════════════════════════════════════════════════════╣"
        ),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print(
            "║  Controls: ← → adjust input  |  ↑ ↓ select pot  |  q or Esc to quit                                        ║"
        ),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print(
            "╚════════════════════════════════════════════════════════════════════════════════════════════════════════════╝"
        ),
        ResetColor,
    )?;

    stdout.flush()?;
    Ok(())
}
