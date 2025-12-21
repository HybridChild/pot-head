mod app_state;
mod color_scheme;
mod pot_display;
mod pot_spec;
mod rendering;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::Result;
use std::io::stdout;
use std::time::Duration;

use app_state::AppState;
use rendering::render;

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), Hide)?;
    Ok(())
}

fn cleanup_terminal() -> Result<()> {
    execute!(stdout(), Show)?;
    disable_raw_mode()?;
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

    while state.running {
        render(&mut state)?;

        // Poll for events with a timeout
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            handle_event(&mut state, event);
        }
    }

    cleanup_terminal()?;
    Ok(())
}
