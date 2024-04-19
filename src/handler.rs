use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use nu_protocol::{
    ast::{CellPath, PathMember},
    ShellError, Span, Value,
};

use crate::{
    app::{App, Mode},
    navigation::Direction,
    nu::value::transpose,
};

/// the result of a state transition
#[derive(Debug, PartialEq)]
pub enum TransitionResult {
    Quit,
    Continue,
    Return(Value),
    Mutate(Value, CellPath),
    Error(String),
}

impl TransitionResult {
    #[cfg(test)]
    fn is_quit(&self) -> bool {
        matches!(self, Self::Quit | Self::Return(_))
    }
}

impl App {
    /// Handles the key events and updates the state of [`App`].
    #[allow(clippy::collapsible_if)]
    pub fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        half_page: usize,
    ) -> Result<TransitionResult, ShellError> {
        let config = &self.config;

        match self.mode {
            Mode::Normal => {
                if key_event.code.ge(&KeyCode::Char('0')) && key_event.code.le(&KeyCode::Char('9'))
                {
                    self.mode = Mode::Waiting(match key_event.code {
                        KeyCode::Char('0') => 0,
                        KeyCode::Char('1') => 1,
                        KeyCode::Char('2') => 2,
                        KeyCode::Char('3') => 3,
                        KeyCode::Char('4') => 4,
                        KeyCode::Char('5') => 5,
                        KeyCode::Char('6') => 6,
                        KeyCode::Char('7') => 7,
                        KeyCode::Char('8') => 8,
                        KeyCode::Char('9') => 9,
                        _ => unreachable!(),
                    });
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.half_page_down {
                    // TODO: add a margin to the bottom
                    self.go_up_or_down_in_data(Direction::Down(half_page));
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.half_page_up {
                    // TODO: add a margin to the top
                    self.go_up_or_down_in_data(Direction::Up(half_page));
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.goto_bottom {
                    self.go_up_or_down_in_data(Direction::Bottom);
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.goto_top {
                    self.go_up_or_down_in_data(Direction::Top);
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.quit {
                    return Ok(TransitionResult::Quit);
                } else if key_event == config.keybindings.insert {
                    self.enter_editor();
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.peek {
                    self.mode = Mode::Peeking;
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.down {
                    self.go_up_or_down_in_data(Direction::Down(1));
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.up {
                    self.go_up_or_down_in_data(Direction::Up(1));
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.right {
                    self.go_deeper_in_data();
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.left {
                    self.go_back_in_data();
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.transpose {
                    let mut path = self.position.clone();
                    path.members.pop();

                    let view = self.value_under_cursor(Some(path.clone()));
                    let transpose = transpose(&view);

                    if transpose != view {
                        match &transpose {
                            Value::Record { val: rec, .. } => {
                                let cols = rec.columns().cloned().collect::<Vec<_>>();

                                // NOTE: app.position.members should never be empty by construction
                                *self.position.members.last_mut().unwrap() = PathMember::String {
                                    val: cols.first().unwrap_or(&"".to_string()).to_string(),
                                    span: Span::unknown(),
                                    optional: cols.is_empty(),
                                };
                            }
                            _ => {
                                // NOTE: app.position.members should never be empty by construction
                                *self.position.members.last_mut().unwrap() = PathMember::Int {
                                    val: 0,
                                    span: Span::unknown(),
                                    optional: false,
                                };
                            }
                        }
                        return Ok(TransitionResult::Mutate(transpose, path));
                    }

                    return Ok(TransitionResult::Continue);
                }
            }
            Mode::Waiting(n) => {
                if key_event.code.ge(&KeyCode::Char('0')) && key_event.code.le(&KeyCode::Char('9'))
                {
                    let u = match key_event.code {
                        KeyCode::Char('0') => 0,
                        KeyCode::Char('1') => 1,
                        KeyCode::Char('2') => 2,
                        KeyCode::Char('3') => 3,
                        KeyCode::Char('4') => 4,
                        KeyCode::Char('5') => 5,
                        KeyCode::Char('6') => 6,
                        KeyCode::Char('7') => 7,
                        KeyCode::Char('8') => 8,
                        KeyCode::Char('9') => 9,
                        _ => unreachable!(),
                    };
                    self.mode = Mode::Waiting(n * 10 + u);
                    return Ok(TransitionResult::Continue);
                } else if key_event.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.down {
                    self.mode = Mode::Normal;
                    self.go_up_or_down_in_data(Direction::Down(n));
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.up {
                    self.mode = Mode::Normal;
                    self.go_up_or_down_in_data(Direction::Up(n));
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.navigation.goto_line {
                    self.mode = Mode::Normal;
                    self.go_up_or_down_in_data(Direction::At(n.saturating_sub(1)));
                    return Ok(TransitionResult::Continue);
                }
            }
            Mode::Insert => {
                if key_event == config.keybindings.normal {
                    self.mode = Mode::Normal;
                    return Ok(TransitionResult::Continue);
                }

                match self.editor.handle_key(&key_event.code) {
                    Some(Some(v)) => {
                        self.mode = Mode::Normal;
                        return Ok(TransitionResult::Mutate(v, self.position.clone()));
                    }
                    Some(None) => {
                        self.mode = Mode::Normal;
                        return Ok(TransitionResult::Continue);
                    }
                    None => return Ok(TransitionResult::Continue),
                }
            }
            Mode::Peeking => {
                if key_event == config.keybindings.quit {
                    return Ok(TransitionResult::Quit);
                } else if key_event == config.keybindings.normal {
                    self.mode = Mode::Normal;
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.peeking.all {
                    return Ok(TransitionResult::Return(self.value.clone()));
                } else if key_event == config.keybindings.peeking.view {
                    self.position.members.pop();
                    return Ok(TransitionResult::Return(self.value_under_cursor(None)));
                } else if key_event == config.keybindings.peeking.under {
                    return Ok(TransitionResult::Return(self.value_under_cursor(None)));
                } else if key_event == config.keybindings.peeking.cell_path {
                    return Ok(TransitionResult::Return(Value::cell_path(
                        self.position.clone(),
                        Span::unknown(),
                    )));
                }
            }
            Mode::Bottom => {
                if key_event == config.keybindings.quit {
                    return Ok(TransitionResult::Quit);
                } else if key_event == config.keybindings.navigation.left {
                    self.mode = Mode::Normal;
                    return Ok(TransitionResult::Continue);
                } else if key_event == config.keybindings.peek {
                    return Ok(TransitionResult::Return(self.value_under_cursor(None)));
                }
            }
        }

        Ok(TransitionResult::Continue)
    }
}

/// represent a [`KeyEvent`] as a simple string
pub fn repr_key(key: &KeyEvent) -> String {
    let code = match key.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Left => char::from_u32(0x2190).unwrap().into(),
        KeyCode::Up => char::from_u32(0x2191).unwrap().into(),
        KeyCode::Right => char::from_u32(0x2192).unwrap().into(),
        KeyCode::Down => char::from_u32(0x2193).unwrap().into(),
        KeyCode::Esc => "<esc>".into(),
        KeyCode::Enter => char::from_u32(0x23ce).unwrap().into(),
        KeyCode::Backspace => char::from_u32(0x232b).unwrap().into(),
        KeyCode::Delete => char::from_u32(0x2326).unwrap().into(),
        _ => "??".into(),
    };

    match key.modifiers {
        KeyModifiers::NONE => code,
        KeyModifiers::CONTROL => format!("<c-{}>", code),
        _ => "??".into(),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyEvent;
    use nu_protocol::{
        ast::{CellPath, PathMember},
        record, Span, Value,
    };

    use super::{repr_key, App, TransitionResult};
    use crate::{
        app::Mode,
        config::Config,
        nu::cell_path::{to_path_member_vec, PM},
    };

    /// {
    ///     l: ["my", "list", "elements"],
    ///     r: {a: 1, b: 2},
    ///     s: "some string",
    ///     i: 123,
    /// }
    fn test_value() -> Value {
        Value::test_record(record! {
            "l" => Value::test_list(vec![
                Value::test_string("my"),
                Value::test_string("list"),
                Value::test_string("elements"),
            ]),
            "r" => Value::test_record(record! {
                "a" => Value::test_int(1),
                "b" => Value::test_int(2),
            }
            ),
            "s" => Value::test_string("some string"),
            "i" => Value::test_int(123),
        })
    }

    #[test]
    fn switch_modes() {
        let mut app = App::from_value(Value::test_string("foo"));
        let keybindings = app.config.clone().keybindings;

        assert!(app.mode == Mode::Normal);

        // INSERT -> PEEKING: not allowed
        // PEEKING -> INSERT: not allowed
        let transitions = vec![
            (keybindings.normal, Mode::Normal),
            (keybindings.insert, Mode::Insert),
            (keybindings.normal, Mode::Normal),
            (keybindings.peek, Mode::Peeking),
            (keybindings.normal, Mode::Normal),
        ];

        for (key, expected_mode) in transitions {
            let mode = app.mode.clone();

            let result = app.handle_key_events(key, 0).unwrap();

            assert!(
                !result.is_quit(),
                "unexpected exit after pressing {} in {}",
                repr_key(&key),
                mode,
            );
            assert!(
                app.mode == expected_mode,
                "expected to be in {} after pressing {} in {}, found {}",
                expected_mode,
                repr_key(&key),
                mode,
                app.mode
            );
        }
    }

    #[test]
    fn quit() {
        let mut app = App::from_value(test_value());
        let keybindings = app.config.clone().keybindings;

        let transitions = vec![
            (keybindings.insert, false),
            (keybindings.quit, false),
            (keybindings.normal, false),
            (keybindings.quit, true),
            (keybindings.peek, false),
            (keybindings.quit, true),
        ];

        for (key, exit) in transitions {
            let mode = app.mode.clone();

            // NOTE: yeah this is a bit clunky...
            if app.mode == Mode::Insert {
                app.editor.set_width(10);
            }

            let result = app.handle_key_events(key, 0).unwrap();

            if exit {
                assert!(
                    result.is_quit(),
                    "expected to quit after pressing {} in {} mode",
                    repr_key(&key),
                    mode
                );
            } else {
                assert!(
                    !result.is_quit(),
                    "expected NOT to quit after pressing {} in {} mode",
                    repr_key(&key),
                    mode
                );
            }
        }
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
        let mut app = App::from_value(test_value());
        let nav = app.config.clone().keybindings.navigation;

        assert!(!app.is_at_bottom());
        assert_eq!(app.position.members, to_path_member_vec(&[PM::S("l")]));

        let transitions = vec![
            (nav.up, vec![PM::S("l")], false),
            (nav.down, vec![PM::S("r")], false),
            (nav.left, vec![PM::S("r")], false),
            (nav.right, vec![PM::S("r"), PM::S("a")], false),
            (nav.right, vec![PM::S("r"), PM::S("a")], true),
            (nav.up, vec![PM::S("r"), PM::S("a")], true),
            (nav.down, vec![PM::S("r"), PM::S("a")], true),
            (nav.left, vec![PM::S("r"), PM::S("a")], false),
            (nav.down, vec![PM::S("r"), PM::S("b")], false),
            (nav.right, vec![PM::S("r"), PM::S("b")], true),
            (nav.up, vec![PM::S("r"), PM::S("b")], true),
            (nav.down, vec![PM::S("r"), PM::S("b")], true),
            (nav.left, vec![PM::S("r"), PM::S("b")], false),
            (nav.up, vec![PM::S("r"), PM::S("a")], false),
            (nav.up, vec![PM::S("r"), PM::S("a")], false),
            (nav.left, vec![PM::S("r")], false),
            (nav.down, vec![PM::S("s")], false),
            (nav.left, vec![PM::S("s")], false),
            (nav.right, vec![PM::S("s")], true),
            (nav.up, vec![PM::S("s")], true),
            (nav.down, vec![PM::S("s")], true),
            (nav.left, vec![PM::S("s")], false),
            (nav.down, vec![PM::S("i")], false),
            (nav.left, vec![PM::S("i")], false),
            (nav.right, vec![PM::S("i")], true),
            (nav.up, vec![PM::S("i")], true),
            (nav.down, vec![PM::S("i")], true),
            (nav.left, vec![PM::S("i")], false),
            (nav.up, vec![PM::S("s")], false),
            (nav.up, vec![PM::S("r")], false),
            (nav.up, vec![PM::S("l")], false),
            (nav.left, vec![PM::S("l")], false),
            (nav.right, vec![PM::S("l"), PM::I(0)], false),
            (nav.right, vec![PM::S("l"), PM::I(0)], true),
            (nav.up, vec![PM::S("l"), PM::I(0)], true),
            (nav.down, vec![PM::S("l"), PM::I(0)], true),
            (nav.left, vec![PM::S("l"), PM::I(0)], false),
            (nav.down, vec![PM::S("l"), PM::I(1)], false),
            (nav.right, vec![PM::S("l"), PM::I(1)], true),
            (nav.up, vec![PM::S("l"), PM::I(1)], true),
            (nav.down, vec![PM::S("l"), PM::I(1)], true),
            (nav.left, vec![PM::S("l"), PM::I(1)], false),
            (nav.down, vec![PM::S("l"), PM::I(2)], false),
            (nav.right, vec![PM::S("l"), PM::I(2)], true),
            (nav.up, vec![PM::S("l"), PM::I(2)], true),
            (nav.down, vec![PM::S("l"), PM::I(2)], true),
            (nav.left, vec![PM::S("l"), PM::I(2)], false),
            (nav.up, vec![PM::S("l"), PM::I(1)], false),
            (nav.up, vec![PM::S("l"), PM::I(0)], false),
            (nav.up, vec![PM::S("l"), PM::I(0)], false),
            (nav.left, vec![PM::S("l")], false),
        ];

        for (key, cell_path, bottom) in transitions {
            let expected = to_path_member_vec(&cell_path);
            app.handle_key_events(key, 0).unwrap();

            if bottom {
                assert!(
                    app.is_at_bottom(),
                    "expected to be at the bottom after pressing {}",
                    repr_key(&key)
                );
            } else {
                assert!(
                    !app.is_at_bottom(),
                    "expected NOT to be at the bottom after pressing {}",
                    repr_key(&key)
                );
            }
            assert_eq!(
                app.position.members,
                expected,
                "expected to be at {:?}, found {:?}",
                repr_path_member_vec(&expected),
                repr_path_member_vec(&app.position.members)
            );
        }
    }

    fn run_peeking_scenario(
        transitions: Vec<(KeyEvent, bool, Option<Value>)>,
        config: Config,
        value: Value,
    ) {
        let mut app = App::from_value(value).with_config(config);

        for (key, exit, expected) in transitions {
            let mode = app.mode.clone();

            let result = app.handle_key_events(key, 0).unwrap();

            if exit {
                assert!(
                    result.is_quit(),
                    "expected to peek some data after pressing {} in {} mode",
                    repr_key(&key),
                    mode
                );
            } else {
                assert!(
                    !result.is_quit(),
                    "expected NOT to peek some data after pressing {} in {} mode",
                    repr_key(&key),
                    mode
                );
            }

            match expected {
                Some(value) => match result {
                    TransitionResult::Return(val) => {
                        assert_eq!(
                            value,
                            val,
                            "unexpected data after pressing {} in {} mode",
                            repr_key(&key),
                            mode
                        )
                    }
                    _ => panic!(
                        "did expect output data after pressing {} in {} mode",
                        repr_key(&key),
                        mode
                    ),
                },
                None => {
                    if let TransitionResult::Return(_) = result {
                        panic!(
                            "did NOT expect output data after pressing {} in {} mode",
                            repr_key(&key),
                            mode
                        )
                    }
                }
            }
        }
    }

    #[test]
    fn peek_data() {
        let config = Config::default();
        let keybindings = config.clone().keybindings;

        let value = test_value();

        let peek_all_from_top = vec![
            (keybindings.peek, false, None),
            (keybindings.peeking.all, true, Some(value.clone())),
        ];
        run_peeking_scenario(peek_all_from_top, config.clone(), value.clone());

        let peek_current_from_top = vec![
            (keybindings.peek, false, None),
            (keybindings.peeking.view, true, Some(value.clone())),
        ];
        run_peeking_scenario(peek_current_from_top, config.clone(), value.clone());

        let go_in_the_data_and_peek_all_and_current = vec![
            (keybindings.navigation.down, false, None),
            (keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
            (keybindings.peek, false, None),
            (keybindings.peeking.all, true, Some(value.clone())),
            (
                keybindings.peeking.view,
                true,
                Some(Value::test_record(record! {
                    "a" => Value::test_int(1),
                    "b" => Value::test_int(2),
                })),
            ),
        ];
        run_peeking_scenario(
            go_in_the_data_and_peek_all_and_current,
            config.clone(),
            value.clone(),
        );

        let go_in_the_data_and_peek_under = vec![
            (keybindings.navigation.down, false, None),
            (keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
            (keybindings.peek, false, None),
            (keybindings.peeking.all, true, Some(value.clone())),
            (keybindings.peeking.under, true, Some(Value::test_int(1))),
        ];
        run_peeking_scenario(go_in_the_data_and_peek_under, config.clone(), value.clone());

        let go_in_the_data_and_peek_cell_path = vec![
            (keybindings.navigation.down, false, None), // on {r: {a: 1, b: 2}}
            (keybindings.navigation.right, false, None), // on {a: 1}
            (keybindings.peek, false, None),
            (
                keybindings.peeking.cell_path,
                true,
                Some(Value::test_cell_path(CellPath {
                    members: vec![
                        PathMember::String {
                            val: "r".into(),
                            span: Span::test_data(),
                            optional: false,
                        },
                        PathMember::String {
                            val: "a".into(),
                            span: Span::test_data(),
                            optional: false,
                        },
                    ],
                })),
            ),
        ];
        run_peeking_scenario(
            go_in_the_data_and_peek_cell_path,
            config.clone(),
            value.clone(),
        );

        let peek_at_the_bottom = vec![
            (keybindings.navigation.right, false, None), // on l: ["my", "list", "elements"],
            (keybindings.navigation.right, false, None), // on "my"
            (keybindings.peek, true, Some(Value::test_string("my"))),
        ];
        run_peeking_scenario(peek_at_the_bottom, config.clone(), value);
    }

    #[test]
    fn transpose_the_data() {
        let mut app = App::from_value(Value::test_record(record!(
            "a" => Value::test_int(1),
            "b" => Value::test_int(2),
            "c" => Value::test_int(3),
        )));
        let kmap = app.config.clone().keybindings;

        assert!(!app.is_at_bottom());
        assert_eq!(app.position.members, to_path_member_vec(&[PM::S("a")]));

        let transitions = vec![
            (kmap.navigation.down, vec![PM::S("b")]),
            (kmap.transpose, vec![PM::I(0)]),
            (kmap.navigation.down, vec![PM::I(1)]),
            (kmap.transpose, vec![PM::S("a")]),
        ];

        for (key, cell_path) in transitions {
            let expected = to_path_member_vec(&cell_path);
            if let TransitionResult::Mutate(cell, path) = app.handle_key_events(key, 0).unwrap() {
                app.value = crate::nu::value::mutate_value_cell(&app.value, &path, &cell).unwrap()
            }

            assert!(
                !app.is_at_bottom(),
                "expected NOT to be at the bottom after pressing {}",
                repr_key(&key)
            );

            assert_eq!(
                app.position.members,
                expected,
                "expected to be at {:?}, found {:?}",
                repr_path_member_vec(&expected),
                repr_path_member_vec(&app.position.members)
            );
        }
    }
}
