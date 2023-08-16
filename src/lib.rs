use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, style::Color, Terminal};

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

fn explore(call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
    let config = Config {
        status_bar: StatusBarConfig {
            background: Color::White,
            foreground: Color::Black,
        },
    };

    let mut terminal = setup_terminal().context("setup failed").unwrap();
    run(&mut terminal, input, &config)
        .context("app loop failed")
        .unwrap();
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

enum State {
    Normal,
    Insert,
}

impl State {
    fn default() -> State {
        State::Normal
    }
}

struct StatusBarConfig {
    background: Color,
    foreground: Color,
}

struct Config {
    status_bar: StatusBarConfig,
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<console::Term>>,
    input: &Value,
    config: &Config,
) -> Result<()> {
    let mut state = State::default();

    loop {
        terminal.draw(|frame| render::ui(frame, input, &state, config))?;
        match console::Term::stderr().read_char()? {
            'q' => break,
            'i' => state = State::Insert,
            'n' => state = State::Normal,
            _ => {}
        }
    }
    Ok(())
}

mod render {
    use ratatui::{
        prelude::{Alignment, CrosstermBackend, Rect},
        style::Style,
        widgets::Paragraph,
        Frame,
    };

    use nu_protocol::Value;

    use super::{Config, State};

    pub(super) fn ui(
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        input: &Value,
        state: &State,
        config: &Config,
    ) {
        data(frame, input);
        status_bar(frame, state, config);
    }

    fn data(frame: &mut Frame<CrosstermBackend<console::Term>>, data: &Value) {
        let rect_without_bottom_bar = Rect::new(0, 0, frame.size().width, frame.size().height - 1);

        frame.render_widget(
            Paragraph::new(format!("{:#?}", data)),
            rect_without_bottom_bar,
        );
    }

    fn status_bar(
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        status: &State,
        config: &Config,
    ) {
        let rect_bottom_bar = Rect::new(0, frame.size().height - 1, frame.size().width, 1);

        let status = match status {
            State::Normal => "NORMAL",
            State::Insert => "INSERT",
        };

        frame.render_widget(
            Paragraph::new(status)
                .style(
                    Style::default()
                        .fg(config.status_bar.foreground)
                        .bg(config.status_bar.background),
                )
                .alignment(Alignment::Left),
            rect_bottom_bar,
        );

        frame.render_widget(
            Paragraph::new("q to quit")
                .style(
                    Style::default()
                        .fg(config.status_bar.foreground)
                        .bg(config.status_bar.background),
                )
                .alignment(Alignment::Right),
            rect_bottom_bar,
        );
    }
}
