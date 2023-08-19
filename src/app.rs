//! the higher level application
//!
//! this module mostly handles
//! 1. the main TUI loop
//! 1. the rendering
//! 1. the keybindings
//! 1. the internal state of the application
use anyhow::Result;
use console::Key;
use ratatui::{prelude::CrosstermBackend, Terminal};

use nu_protocol::{
    ast::{CellPath, PathMember},
    ShellError, Span, Value,
};

use super::navigation::Direction;
use super::{config::Config, navigation, tui};

/// the mode in which the application is
#[derive(PartialEq)]
pub(super) enum Mode {
    /// the NORMAL mode is the *navigation* mode, where the user can move around in the data
    Normal,
    /// the INSERT mode lets the user edit cells of the structured data
    Insert,
    /// the PEEKING mode lets the user *peek* data out of the application, to be reused later
    Peeking,
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
            Mode::Peeking => "PEEKING",
        };
        write!(f, "{}", repr)
    }
}

/// the complete state of the application
pub(super) struct State {
    /// the full current path in the data
    pub cell_path: CellPath,
    /// tells whether or not the user is at the bottom of the data or not, used for rendering in
    /// [`tui`]
    pub bottom: bool,
    /// the current [`Mode`]
    pub mode: Mode,
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

/// the result of a state transition
struct TransitionResult {
    /// whether or not to exit the application
    exit: bool,
    /// a potential value to return
    result: Option<Value>,
}

/// run the application
///
/// this function
/// 1. creates the initial [`State`]
/// 1. runs the main application loop
///
/// the application loop
/// 1. renders the TUI with [`tui`]
/// 1. reads the user's input keys and transition the [`State`] accordingly
pub(super) fn run(
    terminal: &mut Terminal<CrosstermBackend<console::Term>>,
    input: &Value,
    config: &Config,
) -> Result<Value> {
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

        let key = console::Term::stderr().read_key()?;
        match transition_state(key, config, &mut state, input)? {
            TransitionResult { exit: true, result } => match result {
                None => break,
                Some(value) => return Ok(value),
            },
            TransitionResult { exit: false, .. } => {}
        }
    }
    Ok(Value::nothing(Span::unknown()))
}

/// perform the state transition based on the key pressed and the previous state
fn transition_state(
    key: Key,
    config: &Config,
    state: &mut State,
    value: &Value,
) -> Result<TransitionResult, ShellError> {
    if key == config.keybindings.quit {
        return Ok(TransitionResult {
            exit: true,
            result: None,
        });
    } else if key == config.keybindings.insert {
        if state.mode == Mode::Normal {
            state.mode = Mode::Insert;
        }
    } else if key == config.keybindings.normal {
        if state.mode == Mode::Insert {
            state.mode = Mode::Normal;
        }
    } else if key == config.keybindings.navigation.down {
        if state.mode == Mode::Normal {
            navigation::go_up_or_down_in_data(state, value, Direction::Down);
        }
    } else if key == config.keybindings.navigation.up {
        if state.mode == Mode::Normal {
            navigation::go_up_or_down_in_data(state, value, Direction::Up);
        }
    } else if key == config.keybindings.navigation.right {
        if state.mode == Mode::Normal {
            navigation::go_deeper_in_data(state, value);
        }
    } else if key == config.keybindings.navigation.left {
        if state.mode == Mode::Normal {
            navigation::go_back_in_data(state);
        }
    } else if key == config.keybindings.peek {
        if state.mode == Mode::Normal {
            state.mode = Mode::Peeking;
        }
    }

    if state.mode == Mode::Peeking {
        if key == config.keybindings.peeking.quit {
            state.mode = Mode::Normal;
        } else if key == config.keybindings.peeking.all {
            return Ok(TransitionResult {
                exit: true,
                result: Some(value.clone()),
            });
        } else if key == config.keybindings.peeking.current {
            state.cell_path.members.pop();
            return Ok(TransitionResult {
                exit: true,
                result: Some(
                    value
                        .clone()
                        .follow_cell_path(&state.cell_path.members, false)?,
                ),
            });
        } else if key == config.keybindings.peeking.under {
            return Ok(TransitionResult {
                exit: true,
                result: Some(
                    value
                        .clone()
                        .follow_cell_path(&state.cell_path.members, false)?,
                ),
            });
        }
    }

    Ok(TransitionResult {
        exit: false,
        result: None,
    })
}
