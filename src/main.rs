use nu_plugin::{serve_plugin, EvaluatedCall, LabeledError, MsgPackSerializer, Plugin};
use nu_plugin_explore::explore;
use nu_protocol::{Category, PluginExample, PluginSignature, SyntaxShape, Type, Value};

/// the main structure of the [Nushell](https://nushell.sh) plugin
struct Explore;

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

fn main() {
    serve_plugin(&mut Explore {}, MsgPackSerializer {})
}
