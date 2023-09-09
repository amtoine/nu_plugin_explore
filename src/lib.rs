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
use nu_protocol::{Record, ShellError, Span, Value};

use app::{App, Mode};
use config::Config;
use event::{Event, EventHandler};
use handler::{handle_key_events, TransitionResult};
use tui::Tui;

pub fn explore(call: &EvaluatedCall, input: Value) -> Result<Value> {
    let empty_custom_config = Value::record(Record::new(), Span::unknown());
    let config = match Config::from_value(call.opt(0)?.unwrap_or(empty_custom_config)) {
        Ok(cfg) => cfg,
        Err(err) => return Err(ShellError::from(err).into()),
    };

    let mut tui = Tui::new(
        Terminal::new(CrosstermBackend::new(io::stderr()))?,
        EventHandler::new(250),
    );
    tui.init()?;

    let mut app = App::from_value(input);

    loop {
        if app.mode == Mode::Insert {
            app.editor.set_width(tui.size()?.width as usize)
        }

        tui.draw(&mut app, &config, None)?;

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match handle_key_events(key_event, &mut app, &config)? {
                        TransitionResult::Quit => break,
                        TransitionResult::Continue => {}
                        TransitionResult::Mutate(cell, path) => {
                            app.value =
                                crate::nu::value::mutate_value_cell(&app.value, &path, &cell)
                        }
                        TransitionResult::Error(error) => {
                            tui.draw(&mut app, &config, Some(&error))?;
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
