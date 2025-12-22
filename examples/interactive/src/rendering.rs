use crate::app_state::AppState;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Result, Write};

// Bar properties
pub const BAR_WIDTH: usize = 64;

pub fn render_bar(
    processed_value: f32,
    physical_value: f32,
    min: f32,
    max: f32,
    width: usize,
    bar_color: Color,
    processed_indicator_color: Color,
    physical_indicator_color: Color,
    threshold_color: Color,
    threshold_positions: &[f32],
) -> String {
    let processed_normalized = ((processed_value - min) / (max - min)).clamp(0.0, 1.0);
    let physical_normalized = ((physical_value - min) / (max - min)).clamp(0.0, 1.0);

    // Total character positions inside the bar (excluding the pipe characters)
    let inner_width = width - 2;

    // Calculate positions for both indicators
    let processed_pos = (processed_normalized * (inner_width - 1) as f32).round() as usize;
    let physical_pos = (physical_normalized * (inner_width - 1) as f32).round() as usize;

    // Calculate threshold positions (already normalized 0.0-1.0)
    let threshold_positions_idx: Vec<usize> = threshold_positions
        .iter()
        .map(|&t| (t.clamp(0.0, 1.0) * (inner_width - 1) as f32).round() as usize)
        .collect();

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

        // Check if we need to render a threshold marker at this position
        let is_threshold = threshold_positions_idx.contains(&i);

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
            bar.push('◯');
            bar.push_str(&set_color(bar_color));
        } else if is_processed_right {
            bar.push_str(&set_color(processed_indicator_color));
            bar.push('>');
            bar.push_str(&set_color(bar_color));
        } else if is_threshold {
            bar.push_str(&set_color(threshold_color));
            bar.push('┊');
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
            "╔════════════════════════════════════════════════════════════════════════╗"
        ),
        MoveTo(0, 2),
        Print(
            "║                      pot-head Interactive Demo                         ║"
        ),
        MoveTo(0, 3),
        Print(
            "╠════════════════════════════════════════════════════════════════════════╣"
        ),
        ResetColor,
        MoveTo(0, 4),
        Print(""),
    )?;

    let mut line = 5;

    // Render noise control section
    queue!(stdout, MoveTo(0, line), Print(""),)?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        SetForegroundColor(Color::Yellow),
        Print("   Noise Level:"),
        ResetColor,
    )?;
    line += 1;

    // Render noise level bar (simple bar without indicators)
    let noise_bar_width = BAR_WIDTH;
    let noise_filled = (state.noise_level * (noise_bar_width - 2) as f32).round() as usize;
    let mut noise_bar = String::with_capacity(noise_bar_width + 50);
    noise_bar.push_str("\x1b[38;2;255;255;0m"); // Yellow
    noise_bar.push('|');
    for i in 0..(noise_bar_width - 2) {
        if i < noise_filled {
            noise_bar.push('█');
        } else {
            noise_bar.push('-');
        }
    }
    noise_bar.push('|');
    noise_bar.push_str("\x1b[0m");

    queue!(
        stdout,
        MoveTo(0, line),
        Print(format!(
            "     {} {:.0}%",
            noise_bar,
            state.noise_level * 100.0
        )),
    )?;
    line += 1;

    // Get noisy input once before the loop to avoid borrow checker issues
    let noisy_input = state.get_noisy_input();

    // Render each pot
    for (index, pot) in state.pots.iter_mut().enumerate() {
        let is_selected = index == state.selected_pot_index;

        // Only update the selected pot with noisy input
        if is_selected {
            pot.update(noisy_input);
        }

        let info = pot.get_render_info();
        let colors = pot.active_color_scheme(is_selected);

        // Add empty line
        queue!(stdout, MoveTo(0, line), Print(""),)?;
        line += 1;

        // Add selection indicator to the label
        let selection_marker = if is_selected { "► " } else { "  " };

        queue!(
            stdout,
            MoveTo(0, line),
            SetForegroundColor(colors.bar_color),
            Print(format!(
                "   {} {}",
                selection_marker,
                info.label,
            )),
            ResetColor,
        )?;
        line += 1;

        queue!(
            stdout,
            MoveTo(0, line),
            Print(format!(
                "     Input  [{} - {}]: {}  (Hysteresis: {})",
                info.input_range.0,
                info.input_range.1,
                info.input_value,
                info.hysteresis_info,
            )),
        )?;
        line += 1;

        queue!(
            stdout,
            MoveTo(0, line),
            Print(format!(
                "     Output [{} - {}]: {}",
                info.output_range.0,
                info.output_range.1,
                info.output_value,
            )),
        )?;
        line += 1;

        // Physical position shows the noisy input for the selected pot, clean input for others
        let physical_position = if is_selected {
            noisy_input
        } else {
            state.normalized_input
        };

        queue!(
            stdout,
            MoveTo(0, line),
            Print(format!(
                "     {}",
                render_bar(
                    info.output_position, // Processed position (normalized output)
                    physical_position,    // Physical position (normalized input with noise)
                    0.0,
                    1.0,
                    BAR_WIDTH,
                    colors.bar_color,
                    colors.processed_indicator_color,
                    colors.physical_indicator_color,
                    colors.threshold_color,
                    &info.threshold_positions
                )
            )),
        )?;
        line += 1;
    }

    // Legend
    queue!(stdout, MoveTo(0, line), Print(""),)?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        SetForegroundColor(Color::Blue),
        Print(
            "╠════════════════════════════════════════════════════════════════════════╣"
        ),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print("║  Legend:  "),
        SetForegroundColor(Color::Rgb { r: 255, g: 165, b: 0 }),
        Print("|"),
        SetForegroundColor(Color::Blue),
        Print(" Physical Input  "),
        SetForegroundColor(Color::Rgb { r: 0, g: 200, b: 255 }),
        Print("<◯>"),
        SetForegroundColor(Color::Blue),
        Print(" Processed Output  "),
        SetForegroundColor(Color::Rgb { r: 150, g: 150, b: 150 }),
        Print("┊"),
        SetForegroundColor(Color::Blue),
        Print(" Hyst threshold     ║"),
    )?;
    line += 1;

    // Footer
    queue!(
        stdout,
        MoveTo(0, line),
        Print(
            "╠════════════════════════════════════════════════════════════════════════╣"
        ),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print(
            "║  ← → adjust input  |  + - noise  |  ↑ ↓ select pot  |  q/Esc quit      ║"
        ),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print(
            "╚════════════════════════════════════════════════════════════════════════╝"
        ),
        ResetColor,
    )?;

    stdout.flush()?;
    Ok(())
}
