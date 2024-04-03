use nu_plugin::{serve_plugin, EvaluatedCall, MsgPackSerializer, Plugin};
use nu_plugin_explore::explore;
use nu_protocol::{
    Category, LabeledError, PluginExample, PluginSignature, ShellError, Type, Value,
};

/// the main structure of the [Nushell](https://nushell.sh) plugin
struct Explore;

impl Plugin for Explore {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("nu_plugin_explore")
            .usage("interactively explore Nushell structured data")
            .input_output_type(Type::Any, Type::Any)
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
        config: &Option<Value>,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        match name {
            "nu_plugin_explore" => match explore(config, input.clone()) {
                Ok(value) => Ok(value),
                Err(err) => {
                    match err.downcast_ref::<ShellError>() {
                        Some(shell_error) => Err(LabeledError::from(shell_error.clone())),
                        None => Err(LabeledError::new(
                            "unexpected internal error").with_label(
                            "could not transform error into ShellError, there was another kind of crash...",
                            call.head)
                        ),
                    }
                }
            },
            _ => Err(LabeledError::new(
                "Plugin call with wrong name signature").with_label(
                "the signature used to call the plugin does not match any name in the plugin signature vector",
                call.head)
            ),
        }
    }
}

fn main() {
    serve_plugin(&mut Explore {}, MsgPackSerializer {})
}
