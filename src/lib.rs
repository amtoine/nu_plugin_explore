#![doc = include_str!("../README.md")]
mod app;
mod config;
mod edit;
mod handler;
mod navigation;
mod nu;
mod terminal;
mod tui;

use anyhow::{Context, Result};

use nu_plugin::EvaluatedCall;
use nu_protocol::{ShellError, Span, Value};

use app::{App, Mode};
use config::Config;
use handler::{transition_state, TransitionResult};
use terminal::{restore as restore_terminal, setup as setup_terminal};

/// the entry point of the `explore` command
///
/// this function
/// 1. parses the config and default to the [`config::Config::default`] otherwise
/// 1. sets the terminal up (see [`terminal::setup`])
/// 1. runs the application (see [`app::run`])
/// 1. restores the terminal (see [`terminal::restore`])
///
/// # running the application
///
/// 1. creates the initial [`App`]
/// 1. runs the main application loop
///
/// the application loop
/// 1. renders the TUI with [`tui`]
/// 1. reads the user's input keys and transition the [`App`] accordingly
pub fn explore(call: &EvaluatedCall, input: &Value) -> Result<Value> {
    let empty_custom_config = Value::record(vec![], vec![], Span::unknown());
    let config = match Config::from_value(call.opt(0).unwrap().unwrap_or(empty_custom_config)) {
        Ok(cfg) => cfg,
        Err(err) => return Err(ShellError::from(err).into()),
    };

    let mut terminal = setup_terminal().context("setup failed").unwrap();

    let mut app = App::from_value(input);
    let mut value = input.clone();

    loop {
        if app.mode == Mode::Insert {
            app.editor
                .set_width(terminal.size().unwrap().width as usize)
        }

        terminal.draw(|frame| tui::render_ui(frame, &value, &app, &config, None))?;

        let key = console::Term::stderr().read_key()?;
        match transition_state(&key, &config, &mut app, &value)? {
            TransitionResult::Quit => break,
            TransitionResult::Continue => {}
            TransitionResult::Edit(val) => {
                value = crate::nu::value::mutate_value_cell(&value, &app.cell_path, &val)
            }
            TransitionResult::Error(error) => {
                terminal
                    .draw(|frame| tui::render_ui(frame, &value, &app, &config, Some(&error)))?;
                let _ = console::Term::stderr().read_key()?;
            }
            TransitionResult::Return(value) => {
                restore_terminal(&mut terminal)
                    .context("restore terminal failed")
                    .unwrap();
                return Ok(value);
            }
        }
    }

    restore_terminal(&mut terminal)
        .context("restore terminal failed")
        .unwrap();

    Ok(Value::nothing(Span::unknown()))
}
