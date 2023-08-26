use crossterm::event::KeyEvent;
use nu_protocol::Value;

use crate::{
    app::{App, AppResult, Mode},
    config::Config, navigation::{self, Direction},
};

/// the result of a state transition
#[derive(Debug, PartialEq)]
pub struct TransitionResult {
    /// whether or not to exit the application
    pub exit: bool,
    /// a potential value to return
    pub result: Option<Value>,
}

impl TransitionResult {
    /// TODO: documentation
    fn quit() -> Self {
        TransitionResult {
            exit: true,
            result: None,
        }
    }

    /// TODO: documentation
    fn next() -> Self {
        TransitionResult {
            exit: false,
            result: None,
        }
    }

    /// TODO: documentation
    fn output(value: &Value) -> Self {
        TransitionResult {
            exit: true,
            result: Some(value.clone()),
        }
    }
}

/// Handles the key events and updates the state of [`App`].
#[allow(clippy::collapsible_if)]
pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    config: &Config,
    value: &Value,
) -> AppResult<TransitionResult> {
    if key_event.code == config.keybindings.quit {
        return Ok(TransitionResult::quit());
    } else if key_event.code == config.keybindings.insert {
        if app.mode == Mode::Normal {
            app.mode = Mode::Insert;
            return Ok(TransitionResult::next());
        }
    } else if key_event.code == config.keybindings.normal {
        if app.mode == Mode::Insert {
            app.mode = Mode::Normal;
            return Ok(TransitionResult::next());
        }
    } else if key_event.code == config.keybindings.navigation.down {
        if app.mode == Mode::Normal {
            navigation::go_up_or_down_in_data(app, value, Direction::Down);
            return Ok(TransitionResult::next());
        }
    } else if key_event.code == config.keybindings.navigation.up {
        if app.mode == Mode::Normal {
            navigation::go_up_or_down_in_data(app, value, Direction::Up);
            return Ok(TransitionResult::next());
        }
    } else if key_event.code == config.keybindings.navigation.right {
        if app.mode == Mode::Normal {
            navigation::go_deeper_in_data(app, value);
            return Ok(TransitionResult::next());
        }
    } else if key_event.code == config.keybindings.navigation.left {
        if app.mode == Mode::Normal {
            navigation::go_back_in_data(app);
            return Ok(TransitionResult::next());
        } else if app.is_at_bottom() {
            app.mode = Mode::Normal;
            return Ok(TransitionResult::next());
        }
    } else if key_event.code == config.keybindings.peek {
        if app.mode == Mode::Normal {
            app.mode = Mode::Peeking;
            return Ok(TransitionResult::next());
        } else if app.is_at_bottom() {
            return Ok(TransitionResult::output(
                &value
                    .clone()
                    .follow_cell_path(&app.cell_path.members, false)?,
            ));
        }
    }

    if app.mode == Mode::Peeking {
        if key_event.code == config.keybindings.normal {
            app.mode = Mode::Normal;
            return Ok(TransitionResult::next());
        } else if key_event.code == config.keybindings.peeking.all {
            return Ok(TransitionResult::output(value));
        } else if key_event.code == config.keybindings.peeking.current {
            app.cell_path.members.pop();
            return Ok(TransitionResult::output(
                &value
                    .clone()
                    .follow_cell_path(&app.cell_path.members, false)?,
            ));
        } else if key_event.code == config.keybindings.peeking.under {
            return Ok(TransitionResult::output(
                &value
                    .clone()
                    .follow_cell_path(&app.cell_path.members, false)?,
            ));
        }
    }

    Ok(TransitionResult::next())
}

// #[cfg(test)]
// mod tests {
//     use console::Key;
//     use nu_protocol::{ast::PathMember, Span, Value};
//
//     use super::{transition_state, App};
//     use crate::{
//         app::Mode,
//         config::{repr_keycode, Config},
//     };
//
//     /// {
//     ///     l: ["my", "list", "elements"],
//     ///     r: {a: 1, b: 2},
//     ///     s: "some string",
//     ///     i: 123,
//     /// }
//     fn test_value() -> Value {
//         Value::test_record(
//             vec!["l", "r", "s", "i"],
//             vec![
//                 Value::test_list(vec![
//                     Value::test_string("my"),
//                     Value::test_string("list"),
//                     Value::test_string("elements"),
//                 ]),
//                 Value::test_record(vec!["a", "b"], vec![Value::test_int(1), Value::test_int(2)]),
//                 Value::test_string("some string"),
//                 Value::test_int(123),
//             ],
//         )
//     }
//
//     #[test]
//     fn switch_modes() {
//         let config = Config::default();
//         let keybindings = config.clone().keybindings;
//
//         let mut state = App::default();
//         let value = test_value();
//
//         assert!(state.mode == Mode::Normal);
//
//         // INSERT -> PEEKING: not allowed
//         // PEEKING -> INSERT: not allowed
//         let transitions = vec![
//             (&keybindings.normal, Mode::Normal),
//             (&keybindings.insert, Mode::Insert),
//             (&keybindings.insert, Mode::Insert),
//             (&keybindings.normal, Mode::Normal),
//             (&keybindings.peek, Mode::Peeking),
//             (&keybindings.normal, Mode::Normal),
//         ];
//
//         for (key, expected_mode) in transitions {
//             let mode = state.mode.clone();
//
//             let result = transition_state(&key, &config, &mut state, &value).unwrap();
//
//             assert!(
//                 !result.exit,
//                 "unexpected exit after pressing {} in {}",
//                 repr_keycode(key),
//                 mode,
//             );
//             assert!(
//                 state.mode == expected_mode,
//                 "expected to be in {} after pressing {} in {}, found {}",
//                 expected_mode,
//                 repr_keycode(key),
//                 mode,
//                 state.mode
//             );
//         }
//     }
//
//     #[test]
//     fn quit() {
//         let config = Config::default();
//         let keybindings = config.clone().keybindings;
//
//         let mut state = App::default();
//         let value = test_value();
//
//         let transitions = vec![
//             (&keybindings.insert, false),
//             (&keybindings.quit, true),
//             (&keybindings.normal, false),
//             (&keybindings.quit, true),
//             (&keybindings.peek, false),
//             (&keybindings.quit, true),
//         ];
//
//         for (key, exit) in transitions {
//             let mode = state.mode.clone();
//
//             let result = transition_state(key, &config, &mut state, &value).unwrap();
//
//             if exit {
//                 assert!(
//                     result.exit,
//                     "expected to quit after pressing {} in {} mode",
//                     repr_keycode(key),
//                     mode
//                 );
//             } else {
//                 assert!(
//                     !result.exit,
//                     "expected NOT to quit after pressing {} in {} mode",
//                     repr_keycode(key),
//                     mode
//                 );
//             }
//         }
//     }
//
//     /// a simplified [`PathMember`] that can be put in a single vector, without being too long
//     enum PM<'a> {
//         // the [`PathMember::String`] variant
//         S(&'a str),
//         // the [`PathMember::Int`] variant
//         I(usize),
//     }
//
//     fn to_path_member_vec(cell_path: Vec<PM>) -> Vec<PathMember> {
//         cell_path
//             .iter()
//             .map(|x| match *x {
//                 PM::S(val) => PathMember::String {
//                     val: val.into(),
//                     span: Span::test_data(),
//                     optional: false,
//                 },
//                 PM::I(val) => PathMember::Int {
//                     val,
//                     span: Span::test_data(),
//                     optional: false,
//                 },
//             })
//             .collect::<Vec<_>>()
//     }
//
//     fn repr_path_member_vec(members: &[PathMember]) -> String {
//         format!(
//             "$.{}",
//             members
//                 .iter()
//                 .map(|m| {
//                     match m {
//                         PathMember::Int { val, .. } => val.to_string(),
//                         PathMember::String { val, .. } => val.to_string(),
//                     }
//                 })
//                 .collect::<Vec<String>>()
//                 .join(".")
//         )
//     }
//
//     #[test]
//     fn navigate_the_data() {
//         let config = Config::default();
//         let nav = config.clone().keybindings.navigation;
//
//         let value = test_value();
//         let mut state = App::from_value(&value);
//
//         assert!(!state.is_at_bottom());
//         assert_eq!(
//             state.cell_path.members,
//             to_path_member_vec(vec![PM::S("l")])
//         );
//
//         let transitions = vec![
//             (&nav.up, vec![PM::S("i")], false),
//             (&nav.up, vec![PM::S("s")], false),
//             (&nav.up, vec![PM::S("r")], false),
//             (&nav.up, vec![PM::S("l")], false),
//             (&nav.down, vec![PM::S("r")], false),
//             (&nav.left, vec![PM::S("r")], false),
//             (&nav.right, vec![PM::S("r"), PM::S("a")], false),
//             (&nav.right, vec![PM::S("r"), PM::S("a")], true),
//             (&nav.up, vec![PM::S("r"), PM::S("a")], true),
//             (&nav.down, vec![PM::S("r"), PM::S("a")], true),
//             (&nav.left, vec![PM::S("r"), PM::S("a")], false),
//             (&nav.down, vec![PM::S("r"), PM::S("b")], false),
//             (&nav.right, vec![PM::S("r"), PM::S("b")], true),
//             (&nav.up, vec![PM::S("r"), PM::S("b")], true),
//             (&nav.down, vec![PM::S("r"), PM::S("b")], true),
//             (&nav.left, vec![PM::S("r"), PM::S("b")], false),
//             (&nav.up, vec![PM::S("r"), PM::S("a")], false),
//             (&nav.up, vec![PM::S("r"), PM::S("b")], false),
//             (&nav.left, vec![PM::S("r")], false),
//             (&nav.down, vec![PM::S("s")], false),
//             (&nav.left, vec![PM::S("s")], false),
//             (&nav.right, vec![PM::S("s")], true),
//             (&nav.up, vec![PM::S("s")], true),
//             (&nav.down, vec![PM::S("s")], true),
//             (&nav.left, vec![PM::S("s")], false),
//             (&nav.down, vec![PM::S("i")], false),
//             (&nav.left, vec![PM::S("i")], false),
//             (&nav.right, vec![PM::S("i")], true),
//             (&nav.up, vec![PM::S("i")], true),
//             (&nav.down, vec![PM::S("i")], true),
//             (&nav.left, vec![PM::S("i")], false),
//             (&nav.down, vec![PM::S("l")], false),
//             (&nav.left, vec![PM::S("l")], false),
//             (&nav.right, vec![PM::S("l"), PM::I(0)], false),
//             (&nav.right, vec![PM::S("l"), PM::I(0)], true),
//             (&nav.up, vec![PM::S("l"), PM::I(0)], true),
//             (&nav.down, vec![PM::S("l"), PM::I(0)], true),
//             (&nav.left, vec![PM::S("l"), PM::I(0)], false),
//             (&nav.down, vec![PM::S("l"), PM::I(1)], false),
//             (&nav.right, vec![PM::S("l"), PM::I(1)], true),
//             (&nav.up, vec![PM::S("l"), PM::I(1)], true),
//             (&nav.down, vec![PM::S("l"), PM::I(1)], true),
//             (&nav.left, vec![PM::S("l"), PM::I(1)], false),
//             (&nav.down, vec![PM::S("l"), PM::I(2)], false),
//             (&nav.right, vec![PM::S("l"), PM::I(2)], true),
//             (&nav.up, vec![PM::S("l"), PM::I(2)], true),
//             (&nav.down, vec![PM::S("l"), PM::I(2)], true),
//             (&nav.left, vec![PM::S("l"), PM::I(2)], false),
//             (&nav.up, vec![PM::S("l"), PM::I(1)], false),
//             (&nav.up, vec![PM::S("l"), PM::I(0)], false),
//             (&nav.up, vec![PM::S("l"), PM::I(2)], false),
//             (&nav.left, vec![PM::S("l")], false),
//         ];
//
//         for (key, cell_path, bottom) in transitions {
//             let expected = to_path_member_vec(cell_path);
//             transition_state(key, &config, &mut state, &value).unwrap();
//
//             if bottom {
//                 assert!(
//                     state.is_at_bottom(),
//                     "expected to be at the bottom after pressing {}",
//                     repr_keycode(key)
//                 );
//             } else {
//                 assert!(
//                     !state.is_at_bottom(),
//                     "expected NOT to be at the bottom after pressing {}",
//                     repr_keycode(key)
//                 );
//             }
//             assert_eq!(
//                 state.cell_path.members,
//                 expected,
//                 "expected to be at {:?}, found {:?}",
//                 repr_path_member_vec(&expected),
//                 repr_path_member_vec(&state.cell_path.members)
//             );
//         }
//     }
//
//     fn run_peeking_scenario(
//         transitions: Vec<(&Key, bool, Option<Value>)>,
//         config: &Config,
//         value: &Value,
//     ) {
//         let mut state = App::from_value(&value);
//
//         for (key, exit, expected) in transitions {
//             let mode = state.mode.clone();
//
//             let result = transition_state(key, &config, &mut state, &value).unwrap();
//
//             if exit {
//                 assert!(
//                     result.exit,
//                     "expected to peek some data after pressing {} in {} mode",
//                     repr_keycode(key),
//                     mode
//                 );
//             } else {
//                 assert!(
//                     !result.exit,
//                     "expected NOT to peek some data after pressing {} in {} mode",
//                     repr_keycode(key),
//                     mode
//                 );
//             }
//
//             match expected {
//                 Some(value) => match result.result {
//                     Some(v) => assert_eq!(
//                         value,
//                         v,
//                         "unexpected data after pressing {} in {} mode",
//                         repr_keycode(key),
//                         mode
//                     ),
//                     None => panic!(
//                         "did expect output data after pressing {} in {} mode",
//                         repr_keycode(key),
//                         mode
//                     ),
//                 },
//                 None => match result.result {
//                     Some(_) => panic!(
//                         "did NOT expect output data after pressing {} in {} mode",
//                         repr_keycode(key),
//                         mode
//                     ),
//                     None => {}
//                 },
//             }
//         }
//     }
//
//     #[test]
//     fn peek_data() {
//         let config = Config::default();
//         let keybindings = config.clone().keybindings;
//
//         let value = test_value();
//
//         let peek_all_from_top = vec![
//             (&keybindings.peek, false, None),
//             (&keybindings.peeking.all, true, Some(value.clone())),
//         ];
//         run_peeking_scenario(peek_all_from_top, &config, &value);
//
//         let peek_current_from_top = vec![
//             (&keybindings.peek, false, None),
//             (&keybindings.peeking.current, true, Some(value.clone())),
//         ];
//         run_peeking_scenario(peek_current_from_top, &config, &value);
//
//         let go_in_the_data_and_peek_all_and_current = vec![
//             (&keybindings.navigation.down, false, None),
//             (&keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
//             (&keybindings.peek, false, None),
//             (&keybindings.peeking.all, true, Some(value.clone())),
//             (
//                 &keybindings.peeking.current,
//                 true,
//                 Some(Value::test_record(
//                     vec!["a", "b"],
//                     vec![Value::test_int(1), Value::test_int(2)],
//                 )),
//             ),
//         ];
//         run_peeking_scenario(go_in_the_data_and_peek_all_and_current, &config, &value);
//
//         let go_in_the_data_and_peek_under = vec![
//             (&keybindings.navigation.down, false, None),
//             (&keybindings.navigation.right, false, None), // on {r: {a: 1, b: 2}}
//             (&keybindings.peek, false, None),
//             (&keybindings.peeking.all, true, Some(value.clone())),
//             (&keybindings.peeking.under, true, Some(Value::test_int(1))),
//         ];
//         run_peeking_scenario(go_in_the_data_and_peek_under, &config, &value);
//
//         let peek_at_the_bottom = vec![
//             (&keybindings.navigation.right, false, None), // on l: ["my", "list", "elements"],
//             (&keybindings.navigation.right, false, None), // on "my"
//             (&keybindings.peek, true, Some(Value::test_string("my"))),
//         ];
//         run_peeking_scenario(peek_at_the_bottom, &config, &value);
//     }
//
//     #[ignore = "data edition is not implemented for now"]
//     #[test]
//     fn edit_cells() {
//         /**/
//     }
// }
