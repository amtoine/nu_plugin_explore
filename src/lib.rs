#![doc = include_str!("../README.md")]
mod app;
mod config;
mod edit;
mod navigation;
mod nu;
mod terminal;
mod tui;

use anyhow::{Context, Result};

use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Span, Value};

use app::{Mode, State};
use config::Config;
use terminal::restore as restore_terminal;
use terminal::setup as setup_terminal;

/// the entry point of the `explore` command
///
/// this function
/// 1. parses the config and default to the [`config::Config::default`] otherwise
/// 1. sets the terminal up (see [`terminal::setup`])
/// 1. runs the application (see [`app::run`])
/// 1. restores the terminal (see [`terminal::restore`])
pub fn explore(call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
    let config = Config::from_value(call.opt(0).unwrap().unwrap_or(Value::record(
        vec![],
        vec![],
        Span::unknown(),
    )))?;

    let mut terminal = setup_terminal().context("setup failed").unwrap();
    let result = app::run(&mut terminal, input, &config).context("app loop failed");
    restore_terminal(&mut terminal)
        .context("restore terminal failed")
        .unwrap();

    match result {
        Ok(res) => Ok(res),
        Err(err) => Err(LabeledError {
            label: "unexpected error".into(),
            msg: err.to_string(),
            span: Some(call.head),
        }),
    }
}
