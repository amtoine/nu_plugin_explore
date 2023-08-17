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
        show_cell_path: true,
        status_bar: StatusBarConfig {
            background: Color::White,
            foreground: Color::Black,
        },
        keybindings: KeyBindingsMap {
            quit: 'q',
            insert: 'i',
            normal: 'n',
            navigation: NavigationBindingsMap {
                left: 'h',
                down: 'j',
                up: 'k',
                right: 'l',
            },
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
    bottom: bool,
    mode: Mode,
}

impl State {
    fn default() -> State {
        State {
            cell_path: CellPath { members: vec![] },
            bottom: false,
            mode: Mode::default(),
        }
    }
}

struct StatusBarConfig {
    background: Color,
    foreground: Color,
}

struct NavigationBindingsMap {
    up: char,
    down: char,
    left: char,
    right: char,
}

struct KeyBindingsMap {
    quit: char,
    insert: char,
    normal: char,
    navigation: NavigationBindingsMap,
}

struct Config {
    status_bar: StatusBarConfig,
    keybindings: KeyBindingsMap,
    show_cell_path: bool,
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
        Value::List { vals, .. } => state.cell_path.members.push(PathMember::Int {
            val: 0,
            span: Span::unknown(),
            optional: vals.is_empty(),
        }),
        Value::Record { cols, .. } => state.cell_path.members.push(PathMember::String {
            val: cols.get(0).unwrap_or(&"".to_string()).into(),
            span: Span::unknown(),
            optional: cols.is_empty(),
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
        } else if char == config.keybindings.navigation.down {
            if state.mode == Mode::Normal {
                go_up_or_down_in_data(&mut state, input, Direction::Down);
            }
        } else if char == config.keybindings.navigation.up {
            if state.mode == Mode::Normal {
                go_up_or_down_in_data(&mut state, input, Direction::Up);
            }
        } else if char == config.keybindings.navigation.right {
            if state.mode == Mode::Normal {
                go_deeper_in_data(&mut state, input);
            }
        } else if char == config.keybindings.navigation.left {
            if state.mode == Mode::Normal {
                go_back_in_data(&mut state);
            }
        }
    }
    Ok(())
}

fn go_up_or_down_in_data(state: &mut State, input: &Value, direction: Direction) {
    if state.bottom {
        return ();
    }

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
        Ok(Value::List { vals, .. }) => state.cell_path.members.push(PathMember::Int {
            val: 0,
            span: Span::unknown(),
            optional: vals.is_empty(),
        }),
        Ok(Value::Record { cols, .. }) => state.cell_path.members.push(PathMember::String {
            val: cols.get(0).unwrap_or(&"".to_string()).into(),
            span: Span::unknown(),
            optional: cols.is_empty(),
        }),
        Err(_) => panic!("unexpected error when following cell path"),
        _ => state.bottom = true,
    }
}

fn go_back_in_data(state: &mut State) {
    if !state.bottom & (state.cell_path.members.len() > 1) {
        state.cell_path.members.pop();
    }
    state.bottom = false;
}

mod tui {
    use ratatui::{
        prelude::{Alignment, CrosstermBackend, Rect},
        style::Style,
        widgets::Paragraph,
        Frame,
    };

    use nu_protocol::ast::PathMember;
    use nu_protocol::Value;

    use super::{Config, Mode, State};

    pub(super) fn render_ui(
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        input: &Value,
        state: &State,
        config: &Config,
    ) {
        render_data(frame, input, state, config);
        if config.show_cell_path {
            render_cell_path(frame, state);
        }
        render_status_bar(frame, state, config);
    }

    fn render_value(value: &Value) -> String {
        match value {
            Value::List { vals, .. } => {
                if vals.len() <= 1 {
                    format!("[list {} item]", vals.len())
                } else {
                    format!("[list {} items]", vals.len())
                }
            }
            Value::Record { cols, .. } => {
                if cols.len() <= 1 {
                    format!("{{record {} field}}", cols.len())
                } else {
                    format!("{{record {} fields}}", cols.len())
                }
            }
            // FIXME: use a proper conversion to string
            value => value.debug_value(),
        }
    }

    fn render_data(
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        data: &Value,
        state: &State,
        config: &Config,
    ) {
        let data_frame_height = if config.show_cell_path {
            frame.size().height - 2
        } else {
            frame.size().height - 1
        };
        let rect_without_bottom_bar = Rect::new(0, 0, frame.size().width, data_frame_height);

        let mut data_path = state.cell_path.members.clone();
        if !state.bottom {
            data_path.pop();
        }

        let data_repr = match data.clone().follow_cell_path(&data_path, false) {
            Err(_) => panic!("unexpected error when following cell path during rendering"),
            Ok(Value::List { vals, .. }) => vals
                .iter()
                .map(render_value)
                .collect::<Vec<String>>()
                .join("\n"),
            Ok(Value::Record { cols, vals, .. }) => cols
                .iter()
                .zip(vals)
                .map(|(col, val)| format!("{}: {}", col, render_value(&val)))
                .collect::<Vec<String>>()
                .join("\n"),
            // FIXME: use a proper conversion to string
            Ok(value) => value.debug_value(),
        };

        frame.render_widget(Paragraph::new(data_repr), rect_without_bottom_bar);
    }

    fn render_cell_path(frame: &mut Frame<CrosstermBackend<console::Term>>, state: &State) {
        let next_to_bottom_bar_rect = Rect::new(0, frame.size().height - 2, frame.size().width, 1);
        let cell_path = format!(
            "cell path: $.{}",
            state
                .cell_path
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

        frame.render_widget(
            Paragraph::new(cell_path).alignment(Alignment::Left),
            next_to_bottom_bar_rect,
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
            Paragraph::new(state.mode.to_string())
                .style(style)
                .alignment(Alignment::Left),
            bottom_bar_rect,
        );

        let hints = match state.mode {
            Mode::Normal => format!(
                "{} to {} | {}{}{}{} to move around",
                config.keybindings.insert,
                Mode::Insert,
                config.keybindings.navigation.left,
                config.keybindings.navigation.down,
                config.keybindings.navigation.up,
                config.keybindings.navigation.right,
            ),
            Mode::Insert => format!(
                "{} to {} | COMING SOON",
                config.keybindings.normal,
                Mode::Normal
            ),
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
