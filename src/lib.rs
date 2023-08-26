pub mod app;
pub mod config;
pub mod event;
pub mod handler;
pub mod navigation;
pub mod tui;
pub mod ui;

use app::{App, AppResult, Mode};
use config::Config;
use event::{Event, EventHandler};
use handler::{handle_key_events, TransitionResult};
use tui::Tui;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, ShellError, Type, Value};
use nu_protocol::{Span, SyntaxShape};

/// the main structure of the [Nushell](https://nushell.sh) plugin
pub struct Explore;

impl Plugin for Explore {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("explore")
            .usage("interactively explore Nushell structured data")
            .input_output_type(Type::Any, Type::Any)
            .optional(
                "config",
                SyntaxShape::Record(vec![]),
                "a config record to configure everything in explore",
            )
            .plugin_examples(vec![
                PluginExample {
                    example: "open Cargo.toml | explore".into(),
                    description: "explore the Cargo.toml file of this project".into(),
                    result: None,
                },
                PluginExample {
                    example: r#"$nu | explore {show_cell_path: false, layout: "compact"}"#.into(),
                    description: "explore `$nu` and set some config options".into(),
                    result: None,
                },
            ])
            .category(Category::Experimental)]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        match name {
            "explore" => match explore(call, input) {
                Ok(value) => Ok(value),
                Err(err) => {
                    match err.downcast_ref::<ShellError>() {
                        Some(shell_error) => Err(LabeledError::from(shell_error.clone())),
                        None => Err(LabeledError {
                            label: "unexpected internal error".into(),
                            msg: "could not transform error into ShellError, there was another kind of crash...".into(),
                            span: Some(call.head),
                        }),
                    }
                }
            },
            _ => Err(LabeledError {
                label: "Plugin call with wrong name signature".into(),
                msg: "the signature used to call the plugin does not match any name in the plugin signature vector".into(),
                span: Some(call.head),
            }),
        }
    }
}

fn explore(call: &EvaluatedCall, input: &Value) -> AppResult<Value> {
    let empty_custom_config = Value::record(vec![], vec![], Span::unknown());
    let config = match Config::from_value(call.opt(0).unwrap().unwrap_or(empty_custom_config)) {
        Ok(cfg) => cfg,
        Err(err) => return Err(Box::new(ShellError::from(err))),
    };

    let mut app = App::from_value(input);

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    loop {
        tui.draw(&mut app, input, &config)?;

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                match handle_key_events(key_event, &mut app, &config, input)? {
                    TransitionResult { exit: true, result } => match result {
                        None => break,
                        Some(value) => return Ok(value),
                    },
                    TransitionResult { exit: false, .. } => {}
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;

    Ok(Value::nothing(Span::unknown()))
}
