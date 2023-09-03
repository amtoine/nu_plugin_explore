//! navigate in the data in all directions
use nu_protocol::{ast::PathMember, Span, Value};

use crate::app::App;

/// specify a vertical direction in which to go in the data
pub enum Direction {
    /// go one row down in the data
    Down,
    /// go one row up in the data
    Up,
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
    if app.is_at_bottom {
        return;
    }

    let direction = match direction {
        Direction::Up => -1,
        Direction::Down => 1,
    };

    let current = app.position.members.pop();

    match app
        .value
        .clone()
        .follow_cell_path(&app.position.members, false)
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
                        let len = vals.len() as i32;
                        let new_index = (val as i32 + direction + len) % len;

                        new_index as usize
                    },
                    span,
                    optional,
                },
                None => panic!("unexpected error when unpacking current cell path"),
                _ => panic!("current should be an integer path member"),
            };
            app.position.members.push(new);
        }
        Ok(Value::Record { val: rec, .. }) => {
            let new = match current {
                Some(PathMember::String {
                    val,
                    span,
                    optional,
                }) => PathMember::String {
                    val: if rec.cols.is_empty() {
                        "".into()
                    } else {
                        let index = rec.cols.iter().position(|x| x == &val).unwrap() as i32;
                        let len = rec.cols.len() as i32;
                        let new_index = (index + direction + len) % len;

                        rec.cols[new_index as usize].clone()
                    },
                    span,
                    optional,
                },
                None => panic!("unexpected error when unpacking current cell path"),
                _ => panic!("current should be an string path member"),
            };
            app.position.members.push(new);
        }
        Err(_) => panic!("unexpected error when following cell path"),
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
    match app
        .value
        .clone()
        .follow_cell_path(&app.position.members, false)
    {
        Ok(Value::List { vals, .. }) => app.position.members.push(PathMember::Int {
            val: 0,
            span: Span::unknown(),
            optional: vals.is_empty(),
        }),
        Ok(Value::Record { val: rec, .. }) => app.position.members.push(PathMember::String {
            val: rec.cols.get(0).unwrap_or(&"".to_string()).into(),
            span: Span::unknown(),
            optional: rec.cols.is_empty(),
        }),
        Err(_) => panic!("unexpected error when following cell path"),
        _ => app.is_at_bottom = true,
    }
}

/// pop one level of depth from the data
///
/// > :bulb: **Note**  
/// > - the state is always marked as *not at the bottom*
/// > - the state *cell path* can have it's last member popped if possible
pub(super) fn go_back_in_data(app: &mut App) {
    if !app.is_at_bottom & (app.position.members.len() > 1) {
        app.position.members.pop();
    }

    app.is_at_bottom = false;
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
            (Direction::Down, 1),
            (Direction::Down, 2),
            (Direction::Down, 0),
            (Direction::Up, 2),
            (Direction::Up, 1),
            (Direction::Up, 0),
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
            (Direction::Down, "b"),
            (Direction::Down, "c"),
            (Direction::Down, "a"),
            (Direction::Up, "c"),
            (Direction::Up, "b"),
            (Direction::Up, "a"),
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

        assert!(!app.is_at_bottom);

        go_deeper_in_data(&mut app);
        assert!(app.is_at_bottom);
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
        app.is_at_bottom = true;

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
