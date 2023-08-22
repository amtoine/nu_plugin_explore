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

use crate::edit::Editor;

use super::navigation::Direction;
use super::{config::Config, navigation, tui};

/// the mode in which the application is
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Mode {
    /// the NORMAL mode is the *navigation* mode, where the user can move around in the data
    Normal,
    /// the INSERT mode lets the user edit cells of the structured data
    Insert,
    /// the PEEKING mode lets the user *peek* data out of the application, to be reused later
    Peeking,
    /// TODO: documentation
    Bottom,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Peeking => "PEEKING",
            Self::Bottom => "BOTTOM",
        };
        write!(f, "{}", repr)
    }
}

/// the complete state of the application
pub(super) struct State {
    /// the full current path in the data
    pub cell_path: CellPath,
    /// the current [`Mode`]
    pub mode: Mode,
    /// the editor to modify the cells of the data
    pub editor: Editor,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cell_path: CellPath { members: vec![] },
            mode: Mode::default(),
            editor: Editor::default(),
        }
    }
}

impl State {
    pub(super) fn from_value(value: &Value) -> Self {
        let mut state = Self::default();
        match value {
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
        }

        state
    }

    /// TODO: documentation
    pub(super) fn is_at_bottom(&self) -> bool {
        matches!(self.mode, Mode::Bottom)
    }

    /// TODO: documentation
    pub(super) fn hit_bottom(&mut self) {
        self.mode = Mode::Bottom;
    }

    fn enter_editor(&mut self, value: &Value) {
        self.mode = Mode::Insert;
        self.editor = Editor::from_value(value);
    }
}

/// the result of a state transition
#[derive(Debug, PartialEq)]
struct TransitionResult {
    /// whether or not to exit the application
    exit: bool,
    /// a potential value to return
    result: Option<Value>,
    /// a potential error to show
    error: Option<String>,
}

impl TransitionResult {
    /// TODO: documentation
    fn quit() -> Self {
        TransitionResult {
            exit: true,
            result: None,
            error: None,
        }
    }

    /// TODO: documentation
    fn next() -> Self {
        TransitionResult {
            exit: false,
            result: None,
            error: None,
        }
    }

    /// TODO: documentation
    fn output(value: &Value) -> Self {
        TransitionResult {
            exit: true,
            result: Some(value.clone()),
            error: None,
        }
    }

    /// TODO: documentation
    fn error(msg: &str) -> Self {
        TransitionResult {
            exit: false,
            result: None,
            error: Some(msg.to_string()),
        }
    }
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
    let mut state = State::from_value(input);
    let mut value = input.clone();

    loop {
        terminal.draw(|frame| tui::render_ui(frame, &value, &state, config))?;

        let key = console::Term::stderr().read_key()?;
        match transition_state(&key, config, &mut state, &value)? {
            TransitionResult {
                exit: true, result, ..
            } => match result {
                None => break,
                Some(value) => return Ok(value),
            },
            TransitionResult {
                exit: false,
                result: Some(val),
                ..
            } => {
                // FIXME: set the value@cell_path to val, not the whole value
                value = val;
                state = State::from_value(&value);
            }
            TransitionResult {
                exit: false,
                result: None,
                ..
            } => {}
        }
    }
    Ok(Value::nothing(Span::unknown()))
}

/// perform the state transition based on the key pressed and the previous state
#[allow(clippy::collapsible_if)]
fn transition_state(
    key: &Key,
    config: &Config,
    state: &mut State,
    value: &Value,
) -> Result<TransitionResult, ShellError> {
    if key == &config.keybindings.quit {
        if state.mode != Mode::Insert {
            return Ok(TransitionResult::quit());
        }
    } else if key == &config.keybindings.insert {
        if state.mode == Mode::Normal {
            let value = &value
                .clone()
                .follow_cell_path(&state.cell_path.members, false)
                .unwrap();

            match value {
                Value::String { .. } => {
                    state.enter_editor(value);
                    return Ok(TransitionResult::next());
                }
                _ => return Ok(TransitionResult::error("cannot edit non-string cells")),
            }
        }
    } else if key == &config.keybindings.normal {
        if state.mode == Mode::Insert {
            state.mode = Mode::Normal;
            return Ok(TransitionResult::next());
        }
    } else if key == &config.keybindings.navigation.down {
        if state.mode == Mode::Normal {
            navigation::go_up_or_down_in_data(state, value, Direction::Down);
            return Ok(TransitionResult::next());
        }
    } else if key == &config.keybindings.navigation.up {
        if state.mode == Mode::Normal {
            navigation::go_up_or_down_in_data(state, value, Direction::Up);
            return Ok(TransitionResult::next());
        }
    } else if key == &config.keybindings.navigation.right {
        if state.mode == Mode::Normal {
            navigation::go_deeper_in_data(state, value);
            return Ok(TransitionResult::next());
        }
    } else if key == &config.keybindings.navigation.left {
        if state.mode == Mode::Normal {
            navigation::go_back_in_data(state);
            return Ok(TransitionResult::next());
        } else if state.is_at_bottom() {
            state.mode = Mode::Normal;
            return Ok(TransitionResult::next());
        }
    } else if key == &config.keybindings.peek {
        if state.mode == Mode::Normal {
            state.mode = Mode::Peeking;
            return Ok(TransitionResult::next());
        } else if state.is_at_bottom() {
            return Ok(TransitionResult::output(
                &value
                    .clone()
                    .follow_cell_path(&state.cell_path.members, false)?,
            ));
        }
    }

    if state.mode == Mode::Peeking {
        if key == &config.keybindings.normal {
            state.mode = Mode::Normal;
            return Ok(TransitionResult::next());
        } else if key == &config.keybindings.peeking.all {
            return Ok(TransitionResult::output(value));
        } else if key == &config.keybindings.peeking.current {
            state.cell_path.members.pop();
            return Ok(TransitionResult::output(
                &value
                    .clone()
                    .follow_cell_path(&state.cell_path.members, false)?,
            ));
        } else if key == &config.keybindings.peeking.under {
            return Ok(TransitionResult::output(
                &value
                    .clone()
                    .follow_cell_path(&state.cell_path.members, false)?,
            ));
        }
    }

    if state.mode == Mode::Insert {
        match key {
            Key::ArrowLeft => {
                state.editor.move_cursor_left();
                return Ok(TransitionResult::next());
            }
            Key::ArrowRight => {
                state.editor.move_cursor_right();
                return Ok(TransitionResult::next());
            }
            Key::Char(c) => {
                state.editor.enter_char(*c);
                return Ok(TransitionResult::next());
            }
            Key::Backspace => {
                state.editor.delete_char();
                return Ok(TransitionResult::next());
            }
            Key::Enter => {
                state.mode = Mode::Normal;

                let edited_cell = Value::String {
                    val: state.editor.buffer.clone(),
                    span: Span::unknown(),
                };

                return Ok(TransitionResult {
                    exit: false,
                    result: Some(edited_cell),
                    error: None,
                });
            }
            Key::Escape => {
                state.mode = Mode::Normal;
                return Ok(TransitionResult::next());
            }
            _ => {}
        }
    }

    Ok(TransitionResult::next())
}

#[cfg(test)]
mod tests {
    use console::Key;
    use nu_protocol::{ast::PathMember, Span, Value};

    use super::{transition_state, State};
    use crate::{
        app::Mode,
        config::{repr_keycode, Config},
    };

    /// {
    ///     l: ["my", "list", "elements"],
    ///     r: {a: 1, b: 2},
    ///     s: "some string",
    ///     i: 123,
    /// }
    fn test_value() -> Value {
        Value::test_record(
            vec!["l", "r", "s", "i"],
            vec![
                Value::test_list(vec![
                    Value::test_string("my"),
                    Value::test_string("list"),
                    Value::test_string("elements"),
                ]),
                Value::test_record(vec!["a", "b"], vec![Value::test_int(1), Value::test_int(2)]),
                Value::test_string("some string"),
                Value::test_int(123),
            ],
        )
    }

    #[test]
    fn switch_modes() {
        let config = Config::default();
        let keybindings = config.clone().keybindings;

        let mut state = State::default();
        let value = test_value();

        assert!(state.mode == Mode::Normal);

        // INSERT -> PEEKING: not allowed
        // PEEKING -> INSERT: not allowed
        let transitions = vec![
            (&keybindings.normal, Mode::Normal),
            (&keybindings.insert, Mode::Insert),
            (&keybindings.insert, Mode::Insert),
            (&keybindings.normal, Mode::Normal),
            (&keybindings.peek, Mode::Peeking),
            (&keybindings.normal, Mode::Normal),
        ];

        for (key, expected_mode) in transitions {
            let mode = state.mode.clone();

            let result = transition_state(&key, &config, &mut state, &value).unwrap();

            assert!(
                !result.exit,
                "unexpected exit after pressing {} in {}",
                repr_keycode(key),
                mode,
            );
            assert!(
                state.mode == expected_mode,
                "expected to be in {} after pressing {} in {}, found {}",
                expected_mode,
                repr_keycode(key),
                mode,
                state.mode
            );
        }
    }

    #[test]
    fn quit() {
        let config = Config::default();
        let keybindings = config.clone().keybindings;

        let mut state = State::default();
        let value = test_value();

        let transitions = vec![
            (&keybindings.insert, false),
            (&keybindings.quit, true),
            (&keybindings.normal, false),
            (&keybindings.quit, true),
            (&keybindings.peek, false),
            (&keybindings.quit, true),
        ];

        for (key, exit) in transitions {
            let mode = state.mode.clone();

            let result = transition_state(key, &config, &mut state, &value).unwrap();

            if exit {
                assert!(
                    result.exit,
                    "expected to quit after pressing {} in {} mode",
                    repr_keycode(key),
                    mode
                );
            } else {
                assert!(
                    !result.exit,
                    "expected NOT to quit after pressing {} in {} mode",
                    repr_keycode(key),
                    mode
                );
            }
        }
    }

    /// a simplified [`PathMember`] that can be put in a single vector, without being too long
    enum PM<'a> {
        // the [`PathMember::String`] variant
        S(&'a str),
        // the [`PathMember::Int`] variant
        I(usize),
    }

    fn to_path_member_vec(cell_path: Vec<PM>) -> Vec<PathMember> {
        cell_path
            .iter()
            .map(|x| match *x {
                PM::S(val) => PathMember::String {
                    val: val.into(),
                    span: Span::test_data(),
                    optional: false,
                },
                PM::I(val) => PathMember::Int {
                    val,
                    span: Span::test_data(),
                    optional: false,
                },
            })
            .collect::<Vec<_>>()
    }

    fn repr_path_member_vec(members: &[PathMember]) -> String {
        format!(
            "$.{}",
            members
                .iter()
                .map(|m| {
                    match m {
                        PathMember::Int { val, .. } => val.to_string(),
                        PathMember::String { val, .. } => val.to_string(),
                    }
                })
                .collect::<Vec<String>>()
                .join(".")
        )
    }

    #[test]
    fn navigate_the_data() {
        let config = Config::default();
        let nav = config.clone().keybindings.navigation;

        let value = test_value();
        let mut state = State::from_value(&value);

        assert!(!state.is_at_bottom());
        assert_eq!(
            state.cell_path.members,
            to_path_member_vec(vec![PM::S("l")])
        );

        let transitions = vec![
            (&nav.up, vec![PM::S("i")], false),
            (&nav.up, vec![PM::S("s")], false),
            (&nav.up, vec![PM::S("r")], false),
            (&nav.up, vec![PM::S("l")], false),
            (&nav.down, vec![PM::S("r")], false),
            (&nav.left, vec![PM::S("r")], false),
            (&nav.right, vec![PM::S("r"), PM::S("a")], false),
            (&nav.right, vec![PM::S("r"), PM::S("a")], true),
            (&nav.up, vec![PM::S("r"), PM::S("a")], true),
            (&nav.down, vec![PM::S("r"), PM::S("a")], true),
            (&nav.left, vec![PM::S("r"), PM::S("a")], false),
            (&nav.down, vec![PM::S("r"), PM::S("b")], false),
            (&nav.right, vec![PM::S("r"), PM::S("b")], true),
            (&nav.up, vec![PM::S("r"), PM::S("b")], true),
            (&nav.down, vec![PM::S("r"), PM::S("b")], true),
            (&nav.left, vec![PM::S("r"), PM::S("b")], false),
            (&nav.up, vec![PM::S("r"), PM::S("a")], false),
            (&nav.up, vec![PM::S("r"), PM::S("b")], false),
            (&nav.left, vec![PM::S("r")], false),
            (&nav.down, vec![PM::S("s")], false),
            (&nav.left, vec![PM::S("s")], false),
            (&nav.right, vec![PM::S("s")], true),
            (&nav.up, vec![PM::S("s")], true),
            (&nav.down, vec![PM::S("s")], true),
            (&nav.left, vec![PM::S("s")], false),
            (&nav.down, vec![PM::S("i")], false),
            (&nav.left, vec![PM::S("i")], false),
            (&nav.right, vec![PM::S("i")], true),
            (&nav.up, vec![PM::S("i")], true),
            (&nav.down, vec![PM::S("i")], true),
            (&nav.left, vec![PM::S("i")], false),
            (&nav.down, vec![PM::S("l")], false),
            (&nav.left, vec![PM::S("l")], false),
            (&nav.right, vec![PM::S("l"), PM::I(0)], false),
            (&nav.right, vec![PM::S("l"), PM::I(0)], true),
            (&nav.up, vec![PM::S("l"), PM::I(0)], true),
            (&nav.down, vec![PM::S("l"), PM::I(0)], true),
            (&nav.left, vec![PM::S("l"), PM::I(0)], false),
            (&nav.down, vec![PM::S("l"), PM::I(1)], false),
            (&nav.right, vec![PM::S("l"), PM::I(1)], true),
            (&nav.up, vec![PM::S("l"), PM::I(1)], true),
            (&nav.down, vec![PM::S("l"), PM::I(1)], true),
            (&nav.left, vec![PM::S("l"), PM::I(1)], false),
            (&nav.down, vec![PM::S("l"), PM::I(2)], false),
            (&nav.right, vec![PM::S("l"), PM::I(2)], true),
            (&nav.up, vec![PM::S("l"), PM::I(2)], true),
            (&nav.down, vec![PM::S("l"), PM::I(2)], true),
            (&nav.left, vec![PM::S("l"), PM::I(2)], false),
            (&nav.up, vec![PM::S("l"), PM::I(1)], false),
            (&nav.up, vec![PM::S("l"), PM::I(0)], false),
            (&nav.up, vec![PM::S("l"), PM::I(2)], false),
            (&nav.left, vec![PM::S("l")], false),
        ];

        for (key, cell_path, bottom) in transitions {
            let expected = to_path_member_vec(cell_path);
            transition_state(key, &config, &mut state, &value).unwrap();

            if bottom {
                assert!(
                    state.is_at_bottom(),
                    "expected to be at the bottom after pressing {}",
                    repr_keycode(key)
                );
            } else {
                assert!(
                    !state.is_at_bottom(),
                    "expected NOT to be at the bottom after pressing {}",
                    repr_keycode(key)
                );
            }
            assert_eq!(
                state.cell_path.members,
                expected,
                "expected to be at {:?}, found {:?}",
                repr_path_member_vec(&expected),
                repr_path_member_vec(&state.cell_path.members)
            );
        }
    }

    fn run_peeking_scenario(
        transitions: Vec<(&Key, bool, Option<Value>)>,
        config: &Config,
        value: &Value,
    ) {
        let mut state = State::from_value(&value);

        for (key, exit, expected) in transitions {
            let mode = state.mode.clone();

            let result = transition_state(key, &config, &mut state, &value).unwrap();

            if exit {
                assert!(
                    result.exit,
                    "expected to peek some data after pressing {} in {} mode",
                    repr_keycode(key),
                    mode
                );
            } else {
                assert!(
                    !result.exit,
                    "expected NOT to peek some data after pressing {} in {} mode",
                    repr_keycode(key),
                    mode
                );
            }

            match expected {
                Some(value) => match result.result {
                    Some(v) => assert_eq!(
                        value,
                        v,
                        "unexpected data after pressing {} in {} mode",
                        repr_keycode(key),
                        mode
                    ),
                    None => panic!(
                        "did expect output data after pressing {} in {} mode",
                        repr_keycode(key),
                        mode
                    ),
                },
                None => match result.result {
                    Some(_) => panic!(
                        "did NOT expect output data after pressing {} in {} mode",
                        repr_keycode(key),
                        mode
                    ),
                    None => {}
                },
            }
        }
    }

    #[test]
    fn peek_data() {
        let config = Config::default();
        let keybindings = config.clone().keybindings;

        let value = test_value();

        let peek_all_from_top = vec![
            (&keybindings.peek, false, None),
            (&keybindings.peeking.all, true, Some(value.clone())),
        ];
        run_peeking_scenario(peek_all_from_top, &config, &value);

        let peek_current_from_top = vec![
            (&keybindings.peek, false, None),
            (&keybindings.peeking.current, true, Some(value.clone())),
        ];
        run_peeking_scenario(peek_current_from_top, &config, &value);

        let go_in_the_data_and_peek_all_and_current = vec![
            (&keybindings.navigation.down, false, None),
            (&keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
            (&keybindings.peek, false, None),
            (&keybindings.peeking.all, true, Some(value.clone())),
            (
                &keybindings.peeking.current,
                true,
                Some(Value::test_record(
                    vec!["a", "b"],
                    vec![Value::test_int(1), Value::test_int(2)],
                )),
            ),
        ];
        run_peeking_scenario(go_in_the_data_and_peek_all_and_current, &config, &value);

        let go_in_the_data_and_peek_under = vec![
            (&keybindings.navigation.down, false, None),
            (&keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
            (&keybindings.peek, false, None),
            (&keybindings.peeking.all, true, Some(value.clone())),
            (&keybindings.peeking.under, true, Some(Value::test_int(1))),
        ];
        run_peeking_scenario(go_in_the_data_and_peek_under, &config, &value);

        let peek_at_the_bottom = vec![
            (&keybindings.navigation.right, false, None), // on l: ["my", "list", "elements"],
            (&keybindings.navigation.right, false, None), // on "my"
            (&keybindings.peek, true, Some(Value::test_string("my"))),
        ];
        run_peeking_scenario(peek_at_the_bottom, &config, &value);
    }

    #[ignore = "data edition is not implemented for now"]
    #[test]
    fn edit_cells() {
        /**/
    }
}
