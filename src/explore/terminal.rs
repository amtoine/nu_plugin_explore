//! setup and restore the TUI screen
use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

/// setup the terminal to run the TUI
///
/// this function will
/// 1. enter stderr in an *alternate* screen
/// 1. create a new terminal
///
/// > see [`restore`]
pub(super) fn setup() -> Result<Terminal<CrosstermBackend<console::Term>>> {
    let mut stderr = console::Term::stderr();
    execute!(stderr, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stderr)).context("creating terminal failed")
}

/// restore the terminal to the caller
///
/// this function will
/// 1. leave the alternate screen created by [`setup`]
/// 1. show the cursor back
///
/// > see [`setup`]
pub(super) fn restore(terminal: &mut Terminal<CrosstermBackend<console::Term>>) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}
