#![doc = include_str!("../README.md")]
mod app;
mod config;
mod edit;
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

use nu_protocol::{Span, Value};

use app::{App, Mode};
use config::Config;
use handler::TransitionResult;
use tui::{
    event::{Event, EventHandler},
    Tui,
};

pub fn explore(config: &Value, input: Value) -> Result<Value> {
    let mut tui = Tui::new(
        Terminal::new(CrosstermBackend::new(io::stderr()))?,
        EventHandler::new(250),
    );
    tui.init()?;

    let mut app = App::from_value(input).with_config(Config::from_value(config)?);

    loop {
        if app.mode == Mode::Insert {
            app.editor.set_width(tui.size()?.width as usize)
        }

        tui.draw(&mut app, None)?;

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match app.handle_key_events(key_event, (tui.size()?.height as usize - 5) / 2)? {
                        TransitionResult::Quit => break,
                        TransitionResult::Continue => {}
                        TransitionResult::Mutate(cell, path) => {
                            app.value =
                                crate::nu::value::mutate_value_cell(&app.value, &path, &cell)
                                    .unwrap()
                        }
                        TransitionResult::Error(error) => {
                            tui.draw(&mut app, Some(&error))?;
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
