/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Event handler.
pub mod handler;

mod config;
mod navigation;

use app::{App, AppResult, Mode};
use config::Config;
use event::{Event, EventHandler};
use handler::handle_key_events;
use tui::Tui;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Type, Value};
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
            "explore" => explore(call, input),
            _ => Err(LabeledError {
                label: "Plugin call with wrong name signature".into(),
                msg: "the signature used to call the plugin does not match any name in the plugin signature vector".into(),
                span: Some(call.head),
            }),
        }
    }
}

/// the entry point of the `explore` command
///
/// this function
/// 1. parses the config and default to the [`config::Config::default`] otherwise
/// 1. sets the terminal up (see [`terminal::setup`])
/// 1. runs the application (see [`app::run`])
/// 1. restores the terminal (see [`terminal::restore`])
///
/// run the application
///
/// this function
/// 1. creates the initial [`State`]
/// 1. runs the main application loop
///
/// the application loop
/// 1. renders the TUI with [`tui`]
/// 1. reads the user's input keys and transition the [`State`] accordingly
fn explore(call: &EvaluatedCall, input: &Value) -> AppResult<Value> {
    let config = Config::from_value(call.opt(0).unwrap().unwrap_or(Value::record(
        vec![],
        vec![],
        Span::unknown(),
    )))?;

    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // match transition_state(&key, config, &mut state, input)? {
    //     TransitionResult { exit: true, result } => match result {
    //         None => break,
    //         Some(value) => return Ok(value),
    //     },
    //     TransitionResult { exit: false, .. } => {}
    // }
    // Ok(Value::nothing(Span::unknown()));

    // Exit the user interface.
    tui.exit()?;

    // match result {
    //     Ok(res) => Ok(res),
    //     Err(err) => Err(LabeledError {
    //         label: "unexpected error".into(),
    //         msg: err.to_string(),
    //         span: Some(call.head),
    //     }),
    // }
}
