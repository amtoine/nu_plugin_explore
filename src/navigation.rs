//! navigate in the data in all directions
use nu_protocol::{ast::PathMember, Span, Value};

use crate::app::{App, Mode};

/// specify a vertical direction in which to go in the data
pub enum Direction {
    /// go down in the data
    Down(usize),
    /// go up in the data
    Up(usize),
    /// go to the top of the data, i.e. the first element or the first key
    Top,
    /// go to the bottom of the data, i.e. the last element or the last key
    Bottom,
}

/// go up or down in the data
///
/// depending on the direction (see [`Direction`]), this function will
/// - early return if the user is already at the bottom => this is to avoid the confusing following
/// situation: you are at the bottom of the data, looking at one item in a list, without this early
/// return, you'd be able to scroll the list without seeing it as a whole... confusing, right?
/// - cycle the list indices or the record column names => the index / column will wrap around
///
/// > :bulb: **Note**  
/// > this function will only modify the last element of the state's *cell path* either by
/// > - not doing anything
/// > - poping the last element to know where we are and then pushing back the new element
pub(super) fn go_up_or_down_in_data(app: &mut App, direction: Direction) {
    if app.is_at_bottom() {
        return;
    }

    let current = app
        .position
        .members
        .pop()
        .unwrap_or_else(|| panic!("unexpected error: position is empty"));

    let cell = app
        .value
        .clone()
        .follow_cell_path(&app.position.members, false)
        .unwrap_or_else(|_| {
            panic!(
                "unexpected error when following {:?} in {}",
                app.position.members,
                app.value
                    .to_expanded_string(" ", &nu_protocol::Config::default())
            )
        });

    match cell {
        Value::List { vals, .. } => {
            let new = match current {
                PathMember::Int {
                    val,
                    span,
                    optional,
                } => PathMember::Int {
                    val: if vals.is_empty() {
                        val
                    } else {
                        match direction {
                            Direction::Up(step) => val.saturating_sub(step).max(0),
                            Direction::Down(step) => val.saturating_add(step).min(vals.len() - 1),
                            Direction::Top => 0,
                            Direction::Bottom => vals.len() - 1,
                        }
                    },
                    span,
                    optional,
                },
                _ => panic!("current should be an integer path member"),
            };
            app.position.members.push(new);
        }
        Value::Record { val: rec, .. } => {
            let new = match current {
                PathMember::String {
                    val,
                    span,
                    optional,
                } => {
                    let cols = rec.columns().cloned().collect::<Vec<_>>();

                    PathMember::String {
                        val: if cols.is_empty() {
                            "".into()
                        } else {
                            let index = rec.columns().position(|x| x == &val).unwrap();
                            let new_index = match direction {
                                Direction::Up(step) => index.saturating_sub(step).max(0),
                                Direction::Down(step) => {
                                    index.saturating_add(step).min(cols.len() - 1)
                                }
                                Direction::Top => 0,
                                Direction::Bottom => cols.len() - 1,
                            };

                            cols[new_index].clone()
                        },
                        span,
                        optional,
                    }
                }
                _ => panic!("current should be an string path member"),
            };
            app.position.members.push(new);
        }
        _ => {}
    }
}

/// go one level deeper in the data
///
/// > :bulb: **Note**  
/// > this function will
/// > - push a new *cell path* member to the state if there is more depth ahead
/// > - mark the state as *at the bottom* if the value at the new depth is of a simple type
pub(super) fn go_deeper_in_data(app: &mut App) {
    let cell = app
        .value
        .clone()
        .follow_cell_path(&app.position.members, false)
        .unwrap_or_else(|_| {
            panic!(
                "unexpected error when following {:?} in {}",
                app.position.members,
                app.value
                    .to_expanded_string(" ", &nu_protocol::Config::default())
            )
        });

    match cell {
        Value::List { vals, .. } => app.position.members.push(PathMember::Int {
            val: 0,
            span: Span::unknown(),
            optional: vals.is_empty(),
        }),
        Value::Record { val: rec, .. } => {
            let cols = rec.columns().cloned().collect::<Vec<_>>();

            app.position.members.push(PathMember::String {
                val: cols.first().unwrap_or(&"".to_string()).into(),
                span: Span::unknown(),
                optional: cols.is_empty(),
            })
        }
        _ => app.hit_bottom(),
    }
}

/// pop one level of depth from the data
///
/// > :bulb: **Note**  
/// > - the state is always marked as *not at the bottom*
/// > - the state *cell path* can have it's last member popped if possible
pub(super) fn go_back_in_data(app: &mut App) {
    if !app.is_at_bottom() & (app.position.members.len() > 1) {
        app.position.members.pop();
    }
    app.mode = Mode::Normal;
}

// TODO: add proper assert error messages
#[cfg(test)]
mod tests {
    use super::{go_back_in_data, go_deeper_in_data, go_up_or_down_in_data, Direction};
    use crate::app::App;
    use nu_protocol::{ast::PathMember, record, Span, Value};

    fn test_string_pathmember(val: impl Into<String>) -> PathMember {
        PathMember::String {
            val: val.into(),
            span: Span::test_data(),
            optional: false,
        }
    }

    fn test_int_pathmember(val: usize) -> PathMember {
        PathMember::Int {
            val,
            span: Span::test_data(),
            optional: false,
        }
    }

    #[test]
    fn go_up_and_down_in_list() {
        let value = Value::test_list(vec![
            Value::test_nothing(),
            Value::test_nothing(),
            Value::test_nothing(),
        ]);
        let mut app = App::from_value(value);

        let sequence = vec![
            (Direction::Down(1), 1),
            (Direction::Down(1), 2),
            (Direction::Down(1), 2),
            (Direction::Up(1), 1),
            (Direction::Up(1), 0),
            (Direction::Up(1), 0),
            (Direction::Top, 0),
            (Direction::Bottom, 2),
            (Direction::Bottom, 2),
            (Direction::Top, 0),
        ];
        for (direction, id) in sequence {
            go_up_or_down_in_data(&mut app, direction);
            let expected = vec![test_int_pathmember(id)];
            assert_eq!(app.position.members, expected);
        }
    }

    #[test]
    fn go_up_and_down_in_record() {
        let value = Value::test_record(record! {
            "a" => Value::test_nothing(),
            "b" => Value::test_nothing(),
            "c" => Value::test_nothing(),
        });
        let mut app = App::from_value(value);

        let sequence = vec![
            (Direction::Down(1), "b"),
            (Direction::Down(1), "c"),
            (Direction::Down(1), "c"),
            (Direction::Up(1), "b"),
            (Direction::Up(1), "a"),
            (Direction::Up(1), "a"),
            (Direction::Top, "a"),
            (Direction::Bottom, "c"),
            (Direction::Bottom, "c"),
            (Direction::Top, "a"),
        ];
        for (direction, id) in sequence {
            go_up_or_down_in_data(&mut app, direction);
            let expected = vec![test_string_pathmember(id)];
            assert_eq!(app.position.members, expected);
        }
    }

    #[test]
    fn go_deeper() {
        let value = Value::test_list(vec![Value::test_record(record! {
            "a" => Value::test_list(vec![Value::test_nothing()]),
        })]);
        let mut app = App::from_value(value);

        let mut expected = vec![test_int_pathmember(0)];
        assert_eq!(app.position.members, expected);

        go_deeper_in_data(&mut app);
        expected.push(test_string_pathmember("a"));
        assert_eq!(app.position.members, expected);

        go_deeper_in_data(&mut app);
        expected.push(test_int_pathmember(0));
        assert_eq!(app.position.members, expected);
    }

    #[test]
    fn hit_bottom() {
        let value = Value::test_nothing();
        let mut app = App::from_value(value);

        assert!(!app.is_at_bottom());

        go_deeper_in_data(&mut app);
        assert!(app.is_at_bottom());
    }

    #[test]
    fn go_back() {
        let value = Value::test_list(vec![Value::test_record(record! {
            "a" => Value::test_list(vec![Value::test_nothing()]),
        })]);
        let mut app = App::from_value(value);
        app.position.members = vec![
            test_int_pathmember(0),
            test_string_pathmember("a"),
            test_int_pathmember(0),
        ];
        app.hit_bottom();

        let mut expected = app.position.members.clone();

        go_back_in_data(&mut app);
        assert_eq!(app.position.members, expected);

        go_back_in_data(&mut app);
        expected.pop();
        assert_eq!(app.position.members, expected);

        go_back_in_data(&mut app);
        expected.pop();
        assert_eq!(app.position.members, expected);

        go_back_in_data(&mut app);
        assert_eq!(app.position.members, expected);
    }
}
