use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, style::Color, Terminal};

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{
    ast::{CellPath, PathMember},
    Category, PluginExample, PluginSignature, Span, Type, Value,
};

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
        keybindings: KeyBindingsMap {
            quit: 'q',
            insert: 'i',
            normal: 'n',
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

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

impl Mode {
    fn default() -> Mode {
        Mode::Normal
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        };
        write!(f, "{}", repr)
    }
}

struct State {
    cell_path: CellPath,
    mode: Mode,
}

impl State {
    fn default() -> State {
        State {
            cell_path: CellPath { members: vec![] },
            mode: Mode::default(),
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = self.mode.to_string()
            + &format!(
                ": {}",
                self.cell_path
                    .members
                    .iter()
                    .map(|m| {
                        match m {
                            PathMember::Int { val, .. } => format!("{}", val).to_string(),
                            PathMember::String { val, .. } => val.to_string(),
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(".")
            );
        write!(f, "{}", repr)
    }
}

struct StatusBarConfig {
    background: Color,
    foreground: Color,
}

struct KeyBindingsMap {
    quit: char,
    insert: char,
    normal: char,
}

struct Config {
    status_bar: StatusBarConfig,
    keybindings: KeyBindingsMap,
}

enum Direction {
    Down,
    Up,
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<console::Term>>,
    input: &Value,
    config: &Config,
) -> Result<()> {
    let mut state = State::default();
    match input {
        Value::List { vals, .. } => {
            let start = if vals.is_empty() { usize::MAX } else { 0 };
            state.cell_path.members.push(PathMember::Int {
                val: start,
                span: Span::unknown(),
                optional: false,
            })
        }
        Value::Record { cols, .. } => state.cell_path.members.push(PathMember::String {
            val: cols.get(0).unwrap_or(&"".to_string()).into(),
            span: Span::unknown(),
            optional: false,
        }),
        _ => {}
    };

    loop {
        terminal.draw(|frame| tui::render_ui(frame, input, &state, config))?;

        let char = console::Term::stderr().read_char()?;
        if char == config.keybindings.quit {
            break;
        } else if char == config.keybindings.insert {
            state.mode = Mode::Insert;
        } else if char == config.keybindings.normal {
            state.mode = Mode::Normal;
        } else if char == 'j' {
            if state.mode == Mode::Normal {
                go_up_or_down_in_data(&mut state, input, Direction::Down);
            }
        } else if char == 'k' {
            if state.mode == Mode::Normal {
                go_up_or_down_in_data(&mut state, input, Direction::Up);
            }
        } else if char == 'l' {
            if state.mode == Mode::Normal {
                go_deeper_in_data(&mut state, input);
            }
        } else if char == 'h' {
            if state.mode == Mode::Normal {
                go_back_in_data(&mut state);
            }
        }
    }
    Ok(())
}

fn go_up_or_down_in_data(state: &mut State, input: &Value, direction: Direction) {
    let direction = match direction {
        Direction::Up => usize::MAX,
        Direction::Down => 1,
    };

    let current = state.cell_path.members.pop();

    match input
        .clone()
        .follow_cell_path(&state.cell_path.members, false)
    {
        Ok(Value::List { vals, .. }) => {
            let new = match current {
                Some(PathMember::Int {
                    val,
                    span,
                    optional,
                }) => PathMember::Int {
                    val: if vals.is_empty() {
                        val
                    } else {
                        (val + direction + vals.len()) % vals.len()
                    },
                    span,
                    optional,
                },
                None => panic!("unexpected error when unpacking current cell path"),
                _ => panic!("current should be an integer path member"),
            };
            state.cell_path.members.push(new);
        }
        Ok(Value::Record { cols, .. }) => {
            let new = match current {
                Some(PathMember::String {
                    val,
                    span,
                    optional,
                }) => PathMember::String {
                    val: if cols.is_empty() {
                        "".into()
                    } else {
                        let index = cols.iter().position(|x| x == &val).unwrap();
                        cols[(index + direction + cols.len()) % cols.len()].clone()
                    },
                    span,
                    optional,
                },
                None => panic!("unexpected error when unpacking current cell path"),
                _ => panic!("current should be an string path member"),
            };
            state.cell_path.members.push(new);
        }
        Err(_) => panic!("unexpected error when following cell path"),
        _ => {}
    }
}

fn go_deeper_in_data(state: &mut State, input: &Value) {
    match input
        .clone()
        .follow_cell_path(&state.cell_path.members, false)
    {
        Ok(Value::List { vals, .. }) => {
            let start = if vals.is_empty() { usize::MAX } else { 0 };
            state.cell_path.members.push(PathMember::Int {
                val: start,
                span: Span::unknown(),
                optional: false,
            })
        }
        Ok(Value::Record { cols, .. }) => state.cell_path.members.push(PathMember::String {
            val: cols.get(0).unwrap_or(&"".to_string()).into(),
            span: Span::unknown(),
            optional: false,
        }),
        Err(_) => panic!("unexpected error when following cell path"),
        _ => {}
    }
}

fn go_back_in_data(state: &mut State) {
    if state.cell_path.members.len() > 1 {
        state.cell_path.members.pop();
    }
}

mod tui {
    use ratatui::{
        prelude::{Alignment, CrosstermBackend, Rect},
        style::Style,
        widgets::Paragraph,
        Frame,
    };

    use nu_protocol::Value;

    use super::{Config, Mode, State};

    pub(super) fn render_ui(
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        input: &Value,
        state: &State,
        config: &Config,
    ) {
        render_data(frame, input);
        render_status_bar(frame, state, config);
    }

    fn render_data(frame: &mut Frame<CrosstermBackend<console::Term>>, data: &Value) {
        let rect_without_bottom_bar = Rect::new(0, 0, frame.size().width, frame.size().height - 1);

        frame.render_widget(
            Paragraph::new(format!("{:#?}", data)),
            rect_without_bottom_bar,
        );
    }

    fn render_status_bar(
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        state: &State,
        config: &Config,
    ) {
        let bottom_bar_rect = Rect::new(0, frame.size().height - 1, frame.size().width, 1);
        let style = Style::default()
            .fg(config.status_bar.foreground)
            .bg(config.status_bar.background);

        frame.render_widget(
            Paragraph::new(state.to_string())
                .style(style)
                .alignment(Alignment::Left),
            bottom_bar_rect,
        );

        let hints = match state.mode {
            Mode::Normal => format!("{} to {}", config.keybindings.insert, Mode::Insert),
            Mode::Insert => format!("{} to {}", config.keybindings.normal, Mode::Normal),
        }
        .to_string();

        frame.render_widget(
            Paragraph::new(hints + &format!(" | {} to quit", config.keybindings.quit))
                .style(style)
                .alignment(Alignment::Right),
            bottom_bar_rect,
        );
    }
}
