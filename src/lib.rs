#![doc = include_str!("../README.md")]
mod app;
mod config;
mod edit;
mod navigation;
mod nu;
mod terminal;
mod tui;

use anyhow::{Context, Result};

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Type, Value};
use nu_protocol::{Span, SyntaxShape};

use app::{Mode, State};
use config::Config;
use terminal::restore as restore_terminal;
use terminal::setup as setup_terminal;

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
fn explore(call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
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
