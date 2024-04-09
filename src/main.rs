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

        // This is needed to make terminal UI work.
        engine.set_foreground(true)?;

        let result = match explore(config, input.clone()) {
            Ok(value) => Ok(value),
            Err(err) => match err.downcast_ref::<ShellError>() {
                Some(shell_error) => Err(LabeledError::from(shell_error.clone())),
                None => Err(LabeledError::new("unexpected internal error").with_label(
                    "could not transform error into ShellError, there was another kind of crash...",
                    call.head,
                )),
            },
        };

        let reset_result = engine.set_foreground(false).map_err(LabeledError::from);

        result.and_then(|value| reset_result.map(|_| value))
    }
}

fn main() {
    serve_plugin(&ExplorePlugin, MsgPackSerializer {})
}
