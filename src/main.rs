use std::io::IsTerminal;

use nu_plugin::{
    serve_plugin, EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand,
    SimplePluginCommand,
};
use nu_plugin_explore::explore;
use nu_protocol::{Example, LabeledError, Record, ShellError, Signature, Span, Type, Value};

struct ExplorePlugin;

impl Plugin for ExplorePlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Explore)]
    }
}

struct Explore;

impl SimplePluginCommand for Explore {
    type Plugin = ExplorePlugin;

    fn name(&self) -> &str {
        "nu_plugin_explore"
    }

    fn usage(&self) -> &str {
        "interactively explore Nushell structured data"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self)).input_output_type(Type::Any, Type::Any)
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["plugin", "explore"]
    }

    fn examples(&self) -> Vec<nu_protocol::Example> {
        vec![
            Example {
                example: "open Cargo.toml | explore",
                description: "explore the Cargo.toml file of this project",
                result: None,
            },
            Example {
                example: r#"$nu | explore {show_cell_path: false, layout: "compact"}"#,
                description: "explore `$nu` and set some config options",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &ExplorePlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let config = engine.get_config()?;

        let default_config = Value::record(Record::new(), Span::unknown());
        let config = config.plugins.get("explore").unwrap_or(&default_config);

        // Double check that stdin is a terminal. If not, the terminal UI may not work properly.
        if !std::io::stdin().is_terminal() {
            return Err(LabeledError::new("Can't start nu_plugin_explore")
                .with_label("must run in a terminal", call.head)
                .with_help(
                    "ensure that you are running in a terminal, and that the plugin is not \
                    communicating over stdio",
                ));
        }

        // This is needed to make terminal UI work.
        let foreground = engine.enter_foreground()?;

        let value = explore(config, input.clone()).map_err(|err| {
            match err.downcast_ref::<ShellError>() {
                Some(shell_error) => LabeledError::from(shell_error.clone()),
                None => LabeledError::new("unexpected internal error").with_label(
                    "could not transform error into ShellError, there was another kind of crash...",
                    call.head,
                ),
            }
        })?;

        foreground.leave()?;

        Ok(value)
    }
}

fn main() {
    serve_plugin(&ExplorePlugin, MsgPackSerializer {})
}
