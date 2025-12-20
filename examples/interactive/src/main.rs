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
    indicator_color: Color,
}

impl ColorScheme {
    fn dimmed(&self) -> ColorScheme {
        ColorScheme {
            bar_color: dim_color(self.bar_color),
            indicator_color: dim_color(self.indicator_color),
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
    dimmed: bool,
}

impl PotDisplay {
    fn new(
        pot: PotHead<u16, f32>,
        label: &'static str,
        range_display: (f32, f32),
        color_scheme: ColorScheme,
        precision: usize,
        dimmed: bool,
    ) -> Self {
        Self {
            pot,
            label,
            range_display,
            color_scheme,
            precision,
            dimmed,
        }
    }

    fn update(&mut self, input: u16) -> f32 {
        self.pot.update(input)
    }

    fn active_color_scheme(&self) -> ColorScheme {
        if self.dimmed {
            self.color_scheme.dimmed()
        } else {
            self.color_scheme
        }
    }
}

struct AppState {
    input_value: u16,
    pots: Vec<PotDisplay>,
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
                indicator_color: Color::Rgb { r: 0, g: 200, b: 255 },
            },
            4,
            false,  // not dimmed
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
                indicator_color: Color::Rgb { r: 255, g: 100, b: 100 },
            },
            2,
            true,  // dimmed to demonstrate the feature
        ));

        Ok(Self {
            input_value: INPUT_MIN,
            pots,
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

fn render_bar(value: f32, min: f32, max: f32, width: usize, bar_color: Color, indicator_color: Color) -> String {
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);

    // The content width is: width - 2 (for the pipes) - 3 (for <o>) - 2 (for the boundary spaces when they exist)
    // But the boundary spaces are conditional, so we need to think of it as:
    // Total available dash positions = width - 2 (pipes) - 3 (<o>)
    let total_dash_positions = width - 2 - 3;

    // Position ranges from 0 to total_dash_positions
    let position = (normalized * total_dash_positions as f32).round() as usize;
    let position = position.min(total_dash_positions);

    let mut bar = String::with_capacity(width + 100); // Extra space for ANSI codes

    // Set bar color
    bar.push_str(&format!("\x1b[38;2;{};{};{}m",
        match bar_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.0,
        match bar_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.1,
        match bar_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.2
    ));

    bar.push('|');

    // Determine if we have boundary spaces
    let has_left_space = position > 0;
    let has_right_space = position < total_dash_positions;

    // Add left space if needed
    if has_left_space {
        bar.push(' ');
    }

    // Calculate actual dash counts
    // When at position 0: no left space, position 0 means 0 dashes before <o>
    // When at position max: has left space, so dashes_before = position - 1
    let dashes_before = if has_left_space { position - 1 } else { position };

    // Add dashes before the indicator
    for _ in 0..dashes_before {
        bar.push('-');
    }

    // Switch to indicator color
    bar.push_str(&format!("\x1b[38;2;{};{};{}m",
        match indicator_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.0,
        match indicator_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.1,
        match indicator_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.2
    ));

    // Add the indicator
    bar.push_str("<|>");

    // Switch back to bar color
    bar.push_str(&format!("\x1b[38;2;{};{};{}m",
        match bar_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.0,
        match bar_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.1,
        match bar_color {
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (255, 255, 255),
        }.2
    ));

    // Add dashes after the indicator
    let dashes_after = if has_right_space {
        total_dash_positions - position - 1
    } else {
        total_dash_positions - position
    };

    for _ in 0..dashes_after {
        bar.push('-');
    }

    // Add right space if needed
    if has_right_space {
        bar.push(' ');
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
            INPUT_MIN as f32,
            INPUT_MAX as f32,
            BAR_WIDTH,
            Color::Rgb { r: 255, g: 255, b: 0 },
            Color::Rgb { r: 255, g: 165, b: 0 }  // Orange indicator
        ))),
    )?;
    line += 1;

    // Render each pot
    for pot_display in &mut state.pots {
        let output = pot_display.update(state.input_value);
        let colors = pot_display.active_color_scheme();

        queue!(
            stdout,
            MoveTo(0, line),
            Print(""),
        )?;
        line += 1;

        queue!(
            stdout,
            MoveTo(0, line),
            SetForegroundColor(colors.bar_color),
            Print(format!(
                "     {} [{} - {}]: Current value: {:.prec$}",
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
                output,
                pot_display.range_display.1.min(pot_display.range_display.0),
                pot_display.range_display.1.max(pot_display.range_display.0),
                BAR_WIDTH,
                colors.bar_color,
                colors.indicator_color
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
        Print("║  Controls: ← → arrows to adjust  |  q or Esc to quit                                                       ║"),
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
