//! Terminal UI entry point: raw-mode setup, event loop, and teardown.

mod app;
mod input;
mod theme;
mod ui;

use std::io;

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;

use app::App;

/// RAII guard that restores the terminal on drop (including panics).
struct TerminalGuard;

impl TerminalGuard {
    fn init() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// Launch the TUI event loop. Returns when the user quits.
pub fn run() {
    // Install a panic hook that restores the terminal before printing the panic.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        default_hook(info);
    }));

    if let Err(e) = run_inner() {
        eprintln!("TUI error: {e}");
    }
}

fn run_inner() -> io::Result<()> {
    let _guard = TerminalGuard::init()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll with 250ms timeout for responsive UI
        if event::poll(std::time::Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            // Windows crossterm fires Press + Release; only handle Press
            if key.kind == KeyEventKind::Press {
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
