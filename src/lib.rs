mod app;
mod config;
mod navigation;
mod terminal;
mod tui;

use anyhow::{Context, Result};
use console::Key;
use ratatui::style::{Color, Modifier};

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Type, Value};

use app::{Mode, State};
use config::{
    BgFgColorConfig, ColorConfig, Config, KeyBindingsMap, NavigationBindingsMap, PeekingBindingsMap,
};
use terminal::restore as restore_terminal;
use terminal::setup as setup_terminal;

pub struct Explore;

impl Plugin for Explore {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("explore")
            .usage("TODO")
            .input_output_type(Type::Any, Type::Any)
            .plugin_examples(vec![PluginExample {
                example: "open Cargo.toml | explore".into(),
                description: "TODO".into(),
                result: None,
            }])
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

fn explore(call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
    let config = Config {
        show_cell_path: true,
        colors: ColorConfig {
            normal: BgFgColorConfig {
                background: Color::Black,
                foreground: Color::White,
            },
            selected: BgFgColorConfig {
                background: Color::White,
                foreground: Color::Black,
            },
            selected_modifier: Modifier::BOLD,
            selected_symbol: "".into(),
            status_bar: BgFgColorConfig {
                background: Color::White,
                foreground: Color::Black,
            },
        },
        keybindings: KeyBindingsMap {
            quit: Key::Char('q'),
            insert: Key::Char('i'),
            normal: Key::Escape,
            navigation: NavigationBindingsMap {
                left: Key::Char('h'),
                down: Key::Char('j'),
                up: Key::Char('k'),
                right: Key::Char('l'),
            },
            peek: Key::Char('p'),
            peeking: PeekingBindingsMap {
                all: Key::Char('a'),
                current: Key::Char('c'),
                under: Key::Char('u'),
                quit: Key::Escape,
            },
        },
    };

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
