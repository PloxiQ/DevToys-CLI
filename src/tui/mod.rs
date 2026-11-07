mod app;
mod ui;

use std::io::{self, IsTerminal, Write};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::error::{Result, ToolError};

pub use app::{App, Focus, ToolState, TOOLS};

pub fn run() -> Result<()> {
    if !io::stdout().is_terminal() {
        return Err(
            ToolError::new("The interactive TUI requires a terminal on stdout.").with_hint(
                "Run `devtoys --help` for CLI usage, or open DevToys in an interactive terminal.",
            ),
        );
    }

    enable_raw_mode().map_err(terminal_error)?;
    if let Err(error) = execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableBracketedPaste,
        Hide
    ) {
        let _ = disable_raw_mode();
        return Err(terminal_error(error));
    }
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(terminal_error)?;
    terminal.clear().map_err(terminal_error)?;

    let mut app = App::new();
    loop {
        terminal
            .draw(|frame| ui::draw(frame, &mut app))
            .map_err(terminal_error)?;

        match event::read().map_err(terminal_error)? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if app.handle_key(key) {
                    break;
                }
            }
            Event::Paste(text) => app.handle_paste(&text),
            _ => {}
        }
    }

    let _ = io::stdout().flush();
    Ok(())
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableBracketedPaste,
            Show
        );
    }
}

fn terminal_error(error: io::Error) -> ToolError {
    ToolError::new(format!("Terminal error: {error}."))
        .with_hint("Make sure you are running DevToys in a modern interactive terminal.")
}
