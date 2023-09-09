use crossterm::event::KeyEvent;

use nu_protocol::{
    ast::{CellPath, PathMember},
    ShellError, Span, Value,
};

use crate::{
    app::{App, Mode},
    config::Config,
    navigation::{self, Direction},
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

/// Handles the key events and updates the state of [`App`].
#[allow(clippy::collapsible_if)]
pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    config: &Config,
) -> Result<TransitionResult, ShellError> {
    match app.mode {
        Mode::Normal => {
            if key_event.code == config.keybindings.quit {
                return Ok(TransitionResult::Quit);
            } else if key_event.code == config.keybindings.insert {
                match app.enter_editor() {
                    Ok(_) => return Ok(TransitionResult::Continue),
                    Err(err) => return Ok(TransitionResult::Error(err)),
                }
            } else if key_event.code == config.keybindings.peek {
                app.mode = Mode::Peeking;
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.navigation.down {
                navigation::go_up_or_down_in_data(app, Direction::Down);
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.navigation.up {
                navigation::go_up_or_down_in_data(app, Direction::Up);
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.navigation.right {
                navigation::go_deeper_in_data(app);
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.navigation.left {
                navigation::go_back_in_data(app);
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.transpose {
                let mut path = app.position.clone();
                path.members.pop();

                let view = app.value.clone().follow_cell_path(&path.members, false)?;
                let transpose = transpose(&view);

                if transpose != view {
                    match transpose.clone() {
                        Value::Record { val: rec, .. } => {
                            *app.position.members.last_mut().unwrap() = PathMember::String {
                                val: rec.cols.get(0).unwrap_or(&"".to_string()).to_string(),
                                span: Span::unknown(),
                                optional: rec.cols.is_empty(),
                            };
                        }
                        _ => {
                            *app.position.members.last_mut().unwrap() = PathMember::Int {
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
        Mode::Insert => {
            if key_event.code == config.keybindings.normal {
                app.mode = Mode::Normal;
                return Ok(TransitionResult::Continue);
            }

            match app.editor.handle_key(&key_event.code) {
                Some(Some(v)) => {
                    app.mode = Mode::Normal;
                    return Ok(TransitionResult::Mutate(v, app.position.clone()));
                }
                Some(None) => {
                    app.mode = Mode::Normal;
                    return Ok(TransitionResult::Continue);
                }
                None => return Ok(TransitionResult::Continue),
            }
        }
        Mode::Peeking => {
            if key_event.code == config.keybindings.quit {
                return Ok(TransitionResult::Quit);
            } else if key_event.code == config.keybindings.normal {
                app.mode = Mode::Normal;
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.peeking.all {
                return Ok(TransitionResult::Return(app.value.clone()));
            } else if key_event.code == config.keybindings.peeking.view {
                app.position.members.pop();
                return Ok(TransitionResult::Return(
                    app.value
                        .clone()
                        .follow_cell_path(&app.position.members, false)?,
                ));
            } else if key_event.code == config.keybindings.peeking.under {
                return Ok(TransitionResult::Return(
                    app.value
                        .clone()
                        .follow_cell_path(&app.position.members, false)?,
                ));
            } else if key_event.code == config.keybindings.peeking.cell_path {
                return Ok(TransitionResult::Return(Value::cell_path(
                    app.position.clone(),
                    Span::unknown(),
                )));
            }
        }
        Mode::Bottom => {
            if key_event.code == config.keybindings.quit {
                return Ok(TransitionResult::Quit);
            } else if key_event.code == config.keybindings.navigation.left {
                app.mode = Mode::Normal;
                return Ok(TransitionResult::Continue);
            } else if key_event.code == config.keybindings.peek {
                return Ok(TransitionResult::Return(
                    app.value
                        .clone()
                        .follow_cell_path(&app.position.members, false)?,
                ));
            }
        }
    }

    Ok(TransitionResult::Continue)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use nu_protocol::{
        ast::{CellPath, PathMember},
        record, Span, Value,
    };

    use super::{handle_key_events, App, TransitionResult};
    use crate::{
        app::Mode,
        config::{repr_keycode, Config},
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
        let config = Config::default();
        let keybindings = config.clone().keybindings;

        let value = Value::test_string("foo");
        let mut app = App::from_value(value);

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

            let result =
                handle_key_events(KeyEvent::new(key, KeyModifiers::empty()), &mut app, &config)
                    .unwrap();

            assert!(
                !result.is_quit(),
                "unexpected exit after pressing {} in {}",
                repr_keycode(&key),
                mode,
            );
            assert!(
                app.mode == expected_mode,
                "expected to be in {} after pressing {} in {}, found {}",
                expected_mode,
                repr_keycode(&key),
                mode,
                app.mode
            );
        }
    }

    #[test]
    fn quit() {
        let config = Config::default();
        let keybindings = config.clone().keybindings;

        let value = test_value();
        let mut app = App::from_value(value);

        let transitions = vec![
            (keybindings.insert, false),
            (keybindings.quit, true),
            (keybindings.normal, false),
            (keybindings.quit, true),
            (keybindings.peek, false),
            (keybindings.quit, true),
        ];

        for (key, exit) in transitions {
            let mode = app.mode.clone();

            let result =
                handle_key_events(KeyEvent::new(key, KeyModifiers::empty()), &mut app, &config)
                    .unwrap();

            if exit {
                assert!(
                    result.is_quit(),
                    "expected to quit after pressing {} in {} mode",
                    repr_keycode(&key),
                    mode
                );
            } else {
                assert!(
                    !result.is_quit(),
                    "expected NOT to quit after pressing {} in {} mode",
                    repr_keycode(&key),
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
        let config = Config::default();
        let nav = config.clone().keybindings.navigation;

        let value = test_value();
        let mut app = App::from_value(value.clone());

        assert!(!app.is_at_bottom());
        assert_eq!(app.position.members, to_path_member_vec(&[PM::S("l")]));

        let transitions = vec![
            (nav.up, vec![PM::S("i")], false),
            (nav.up, vec![PM::S("s")], false),
            (nav.up, vec![PM::S("r")], false),
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
            (nav.up, vec![PM::S("r"), PM::S("b")], false),
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
            (nav.down, vec![PM::S("l")], false),
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
            (nav.up, vec![PM::S("l"), PM::I(2)], false),
            (nav.left, vec![PM::S("l")], false),
        ];

        for (key, cell_path, bottom) in transitions {
            let expected = to_path_member_vec(&cell_path);
            handle_key_events(KeyEvent::new(key, KeyModifiers::empty()), &mut app, &config)
                .unwrap();

            if bottom {
                assert!(
                    app.is_at_bottom(),
                    "expected to be at the bottom after pressing {}",
                    repr_keycode(&key)
                );
            } else {
                assert!(
                    !app.is_at_bottom(),
                    "expected NOT to be at the bottom after pressing {}",
                    repr_keycode(&key)
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
        transitions: Vec<(KeyCode, bool, Option<Value>)>,
        config: &Config,
        value: Value,
    ) {
        let mut app = App::from_value(value);

        for (key, exit, expected) in transitions {
            let mode = app.mode.clone();

            let result =
                handle_key_events(KeyEvent::new(key, KeyModifiers::empty()), &mut app, &config)
                    .unwrap();

            if exit {
                assert!(
                    result.is_quit(),
                    "expected to peek some data after pressing {} in {} mode",
                    repr_keycode(&key),
                    mode
                );
            } else {
                assert!(
                    !result.is_quit(),
                    "expected NOT to peek some data after pressing {} in {} mode",
                    repr_keycode(&key),
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
                            repr_keycode(&key),
                            mode
                        )
                    }
                    _ => panic!(
                        "did expect output data after pressing {} in {} mode",
                        repr_keycode(&key),
                        mode
                    ),
                },
                None => match result {
                    TransitionResult::Return(_) => panic!(
                        "did NOT expect output data after pressing {} in {} mode",
                        repr_keycode(&key),
                        mode
                    ),
                    _ => {}
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
            (keybindings.peek, false, None),
            (keybindings.peeking.all, true, Some(value.clone())),
        ];
        run_peeking_scenario(peek_all_from_top, &config, value.clone());

        let peek_current_from_top = vec![
            (keybindings.peek, false, None),
            (keybindings.peeking.view, true, Some(value.clone())),
        ];
        run_peeking_scenario(peek_current_from_top, &config, value.clone());

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
            &config,
            value.clone(),
        );

        let go_in_the_data_and_peek_under = vec![
            (keybindings.navigation.down, false, None),
            (keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
            (keybindings.peek, false, None),
            (keybindings.peeking.all, true, Some(value.clone())),
            (keybindings.peeking.under, true, Some(Value::test_int(1))),
        ];
        run_peeking_scenario(go_in_the_data_and_peek_under, &config, value.clone());

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
        run_peeking_scenario(go_in_the_data_and_peek_cell_path, &config, value.clone());

        let peek_at_the_bottom = vec![
            (keybindings.navigation.right, false, None), // on l: ["my", "list", "elements"],
            (keybindings.navigation.right, false, None), // on "my"
            (keybindings.peek, true, Some(Value::test_string("my"))),
        ];
        run_peeking_scenario(peek_at_the_bottom, &config, value);
    }
}
