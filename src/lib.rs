use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, widgets::Paragraph, Frame, Terminal};

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Type, Value};

pub struct Explore;

impl Plugin for Explore {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("explore")
            .usage("TODO")
            .input_output_type(Type::Any, Type::Nothing)
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

fn explore(call: &EvaluatedCall, _input: &Value) -> Result<Value, LabeledError> {
    let mut terminal = setup_terminal().context("setup failed").unwrap();
    run(&mut terminal).context("app loop failed").unwrap();
    restore_terminal(&mut terminal)
        .context("restore terminal failed")
        .unwrap();

    Ok(Value::nothing(call.head))
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<console::Term>>> {
    let mut stderr = console::Term::stderr();
    execute!(stderr, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stderr)).context("creating terminal failed")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<console::Term>>) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

fn run(terminal: &mut Terminal<CrosstermBackend<console::Term>>) -> Result<()> {
    loop {
        terminal.draw(render_app)?;
        if console::Term::stderr().read_char()? == 'q' {
            break;
        }
    }
    Ok(())
}

fn render_app(frame: &mut Frame<CrosstermBackend<console::Term>>) {
    let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, frame.size());
}
