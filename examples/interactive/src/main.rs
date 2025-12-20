use pot_head::{Config, PotHead};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Color, SetForegroundColor, ResetColor},
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::{Hide, Show, MoveTo},
};
use std::io::{stdout, Write, Result};
use std::time::Duration;

// Global input range
const INPUT_MIN: u16 = 0;
const INPUT_MAX: u16 = 99;
const STEP_SIZE: u16 = 1;

// Standard potmeter
const OUTPUT_MIN: f32 = 0.0;
const OUTPUT_MAX: f32 = 1.0;

// Reversed polarity potmeter
const REVERSED_MIN: f32 = 100.0;
const REVERSED_MAX: f32 = -100.0;

// Bar properties
const BAR_WIDTH: usize = 100;

#[derive(Clone, Copy)]
struct ColorScheme {
    bar_color: Color,
    processed_indicator_color: Color,
    physical_indicator_color: Color,
}

impl ColorScheme {
    fn dimmed(&self) -> ColorScheme {
        ColorScheme {
            bar_color: dim_color(self.bar_color),
            processed_indicator_color: dim_color(self.processed_indicator_color),
            physical_indicator_color: dim_color(self.physical_indicator_color),
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

struct PotDisplay {
    pot: PotHead<u16, f32>,
    label: &'static str,
    range_display: (f32, f32),
    color_scheme: ColorScheme,
    precision: usize,
    last_output: f32,  // Store the last processed output
}

impl PotDisplay {
    fn new(
        pot: PotHead<u16, f32>,
        label: &'static str,
        range_display: (f32, f32),
        color_scheme: ColorScheme,
        precision: usize,
    ) -> Self {
        // Initialize last_output to the minimum of the range
        let initial_output = range_display.0.min(range_display.1);

        Self {
            pot,
            label,
            range_display,
            color_scheme,
            precision,
            last_output: initial_output,
        }
    }

    fn update(&mut self, input: u16) -> f32 {
        self.last_output = self.pot.update(input);
        self.last_output
    }

    fn active_color_scheme(&self, is_selected: bool) -> ColorScheme {
        if is_selected {
            self.color_scheme
        } else {
            self.color_scheme.dimmed()
        }
    }
}

struct AppState {
    input_value: u16,
    pots: Vec<PotDisplay>,
    selected_pot_index: usize,
    running: bool,
}

impl AppState {
    fn new() -> Result<Self> {
        let mut pots = Vec::new();

        // Standard potmeter
        let config_standard = Config {
            input_min: INPUT_MIN,
            input_max: INPUT_MAX,
            output_min: OUTPUT_MIN,
            output_max: OUTPUT_MAX,
        };

        let pot_standard = PotHead::new(config_standard)
            .map_err(|e| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("PotHead config error: {:?}", e)
            ))?;

        pots.push(PotDisplay::new(
            pot_standard,
            "Standard Pot",
            (OUTPUT_MIN, OUTPUT_MAX),
            ColorScheme {
                bar_color: Color::Rgb { r: 0, g: 255, b: 0 },
                processed_indicator_color: Color::Rgb { r: 0, g: 200, b: 255 },  // Cyan
                physical_indicator_color: Color::Rgb { r: 255, g: 165, b: 0 },   // Orange
            },
            4,
        ));

        // Reversed polarity potmeter
        let config_reversed = Config {
            input_min: INPUT_MIN,
            input_max: INPUT_MAX,
            output_min: REVERSED_MIN,
            output_max: REVERSED_MAX,
        };

        let pot_reversed = PotHead::new(config_reversed)
            .map_err(|e| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("PotHead reversed config error: {:?}", e)
            ))?;

        pots.push(PotDisplay::new(
            pot_reversed,
            "Reversed Pot",
            (REVERSED_MIN, REVERSED_MAX),
            ColorScheme {
                bar_color: Color::Rgb { r: 255, g: 0, b: 255 },
                processed_indicator_color: Color::Rgb { r: 255, g: 100, b: 100 },  // Pink
                physical_indicator_color: Color::Rgb { r: 255, g: 200, b: 0 },     // Yellow
            },
            2,
        ));

        Ok(Self {
            input_value: INPUT_MIN,
            pots,
            selected_pot_index: 0,  // Start with first pot selected
            running: true,
        })
    }

    fn increase_input(&mut self) {
        self.input_value = self.input_value
            .saturating_add(STEP_SIZE)
            .min(INPUT_MAX);
    }

    fn decrease_input(&mut self) {
        self.input_value = self.input_value
            .saturating_sub(STEP_SIZE)
            .max(INPUT_MIN);
    }

    fn select_next_pot(&mut self) {
        if !self.pots.is_empty() {
            self.selected_pot_index = (self.selected_pot_index + 1) % self.pots.len();
        }
    }

    fn select_prev_pot(&mut self) {
        if !self.pots.is_empty() {
            self.selected_pot_index = if self.selected_pot_index == 0 {
                self.pots.len() - 1
            } else {
                self.selected_pot_index - 1
            };
        }
    }
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), Hide)?;
    Ok(())
}

fn cleanup_terminal() -> Result<()> {
    execute!(stdout(), Show, Clear(ClearType::All), MoveTo(0, 0))?;
    disable_raw_mode()?;
    Ok(())
}

fn render_bar(
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

fn render(state: &mut AppState) -> Result<()> {
    let mut stdout = stdout();

    // Start with header
    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(""),
        MoveTo(0, 1),
        SetForegroundColor(Color::Blue),
        Print("╔════════════════════════════════════════════════════════════════════════════════════════════════════════════╗"),
        MoveTo(0, 2),
        Print("║                                        pot-head Interactive Demo                                           ║"),
        MoveTo(0, 3),
        Print("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════╣"),
        ResetColor,
        MoveTo(0, 4),
        Print(""),
    )?;

    let mut line = 5;

    // Render input
    queue!(
        stdout,
        MoveTo(0, line),
        SetForegroundColor(Color::Rgb { r: 255, g: 255, b: 0 }),
        Print(format!("     Input [{} - {}]: Current value: {}", INPUT_MIN, INPUT_MAX, state.input_value)),
        ResetColor,
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print(format!("     {}", render_bar(
            state.input_value as f32,
            state.input_value as f32,  // Physical and processed are the same for raw input
            INPUT_MIN as f32,
            INPUT_MAX as f32,
            BAR_WIDTH,
            Color::Rgb { r: 255, g: 255, b: 0 },
            Color::Rgb { r: 200, g: 200, b: 0 },  // Processed indicator (darker yellow)
            Color::Rgb { r: 255, g: 165, b: 0 }   // Physical indicator (orange)
        ))),
    )?;
    line += 1;

    // Render each pot
    for (index, pot_display) in state.pots.iter_mut().enumerate() {
        let is_selected = index == state.selected_pot_index;

        // Only update the selected pot
        let output = if is_selected {
            pot_display.update(state.input_value)
        } else {
            pot_display.last_output  // Use cached output for non-selected pots
        };

        let colors = pot_display.active_color_scheme(is_selected);

        // Calculate the physical position (normalized input in the output range)
        let input_normalized = (state.input_value - INPUT_MIN) as f32 / (INPUT_MAX - INPUT_MIN) as f32;
        let physical_position = pot_display.range_display.0
            + input_normalized * (pot_display.range_display.1 - pot_display.range_display.0);

        queue!(
            stdout,
            MoveTo(0, line),
            Print(""),
        )?;
        line += 1;

        // Add selection indicator to the label
        let selection_marker = if is_selected { "► " } else { "  " };

        queue!(
            stdout,
            MoveTo(0, line),
            SetForegroundColor(colors.bar_color),
            Print(format!(
                "   {} {} [{} - {}]: Current value: {:.prec$}",
                selection_marker,
                pot_display.label,
                pot_display.range_display.0,
                pot_display.range_display.1,
                output,
                prec = pot_display.precision
            )),
            ResetColor,
        )?;
        line += 1;

        queue!(
            stdout,
            MoveTo(0, line),
            Print(format!("     {}", render_bar(
                output,                 // Processed position (output from PotHead)
                physical_position,      // Physical position (normalized input)
                pot_display.range_display.1.min(pot_display.range_display.0),
                pot_display.range_display.1.max(pot_display.range_display.0),
                BAR_WIDTH,
                colors.bar_color,
                colors.processed_indicator_color,
                colors.physical_indicator_color
            ))),
        )?;
        line += 1;
    }

    // Footer
    queue!(
        stdout,
        MoveTo(0, line),
        Print(""),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        SetForegroundColor(Color::Blue),
        Print("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════╣"),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print("║  Controls: ← → adjust input  |  ↑ ↓ select pot  |  q or Esc to quit                                        ║"),
    )?;
    line += 1;

    queue!(
        stdout,
        MoveTo(0, line),
        Print("╚════════════════════════════════════════════════════════════════════════════════════════════════════════════╝"),
        ResetColor,
    )?;

    stdout.flush()?;
    Ok(())
}

fn handle_event(state: &mut AppState, event: Event) {
    if let Event::Key(KeyEvent { code, .. }) = event {
        match code {
            KeyCode::Left => state.decrease_input(),
            KeyCode::Right => state.increase_input(),
            KeyCode::Up => state.select_prev_pot(),
            KeyCode::Down => state.select_next_pot(),
            KeyCode::Char('q') | KeyCode::Esc => state.running = false,
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let mut state = AppState::new()?;

    setup_terminal()?;

    // Ensure cleanup on panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Initial render
        render(&mut state)?;

        // Main event loop
        while state.running {
            // Poll for events with timeout
            if poll(Duration::from_millis(50))? {
                let event = read()?;
                handle_event(&mut state, event);
                render(&mut state)?;
            }
        }

        Ok::<(), std::io::Error>(())
    }));

    cleanup_terminal()?;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Application panicked"
        )),
    }
}
