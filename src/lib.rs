#![doc = include_str!("../README.md")]
mod app;
mod config;
mod edit;
mod event;
mod handler;
mod navigation;
mod nu;
mod tui;
mod ui;

use anyhow::Result;
use crossterm::event::KeyEventKind;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use nu_plugin::EvaluatedCall;
use nu_protocol::{ShellError, Span, Value};

use app::{App, Mode};
use config::Config;
use event::{Event, EventHandler};
use handler::{handle_key_events, TransitionResult};
use tui::Tui;

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

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let mut app = App::from_value(input);
    let mut value = input.clone();

    loop {
        if app.mode == Mode::Insert {
            app.editor.set_width(tui.size().unwrap().width as usize)
        }

        tui.draw(&mut app, &value, &config, None)?;

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match handle_key_events(key_event, &mut app, &config, &value)? {
                        TransitionResult::Quit => break,
                        TransitionResult::Continue => {}
                        TransitionResult::Edit(val) => {
                            value =
                                crate::nu::value::mutate_value_cell(&value, &app.cell_path, &val)
                        }
                        TransitionResult::Error(error) => {
                            tui.draw(&mut app, &value, &config, Some(&error))?;
                            loop {
                                if let Event::Key(_) = tui.events.next()? {
                                    break;
                                }
                            }
                        }
                        TransitionResult::Return(value) => {
                            tui.exit()?;
                            return Ok(value);
                        }
                    }
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;

    Ok(Value::nothing(Span::unknown()))
}
