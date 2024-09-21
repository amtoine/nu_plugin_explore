use std::io::IsTerminal;

use nu_plugin::{
    serve_plugin, EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand,
    SimplePluginCommand,
};
use nu_plugin_explore::explore;
use nu_protocol::{Example, LabeledError, Record, Signature, Span, Type, Value};

struct ExplorePlugin;

impl Plugin for ExplorePlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Explore)]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
}

struct Explore;

impl SimplePluginCommand for Explore {
    type Plugin = ExplorePlugin;

    fn name(&self) -> &str {
        "nu_plugin_explore"
    }

    fn description(&self) -> &str {
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
                example: "open Cargo.toml | nu_plugin_explore",
                description: "explore the Cargo.toml file of this project",
                result: None,
            },
            Example {
                example: r#"$env.config.plugins.explore = { show_cell_path: false, layout: "compact" }
    $nu | nu_plugin_explore"#,
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

        if !std::io::stdin().is_terminal() {
            return Err(LabeledError::new("Can't start nu_plugin_explore")
                .with_label("must run in a terminal", call.head)
                .with_help(
                    "ensure that you are running in a terminal, and that the plugin is not \
                    communicating over stdio",
                ));
        }

        let foreground = engine.enter_foreground()?;

        let value = explore(config, input.clone()).map_err(|err| {
            match err.downcast_ref::<LabeledError>() {
                Some(err) => err.clone(),
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
