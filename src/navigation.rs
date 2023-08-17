use nu_protocol::{ast::PathMember, Span, Value};

use super::app::State;

pub(super) enum Direction {
    Down,
    Up,
}

pub(super) fn go_up_or_down_in_data(state: &mut State, input: &Value, direction: Direction) {
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

pub(super) fn go_deeper_in_data(state: &mut State, input: &Value) {
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

pub(super) fn go_back_in_data(state: &mut State) {
    if !state.bottom & (state.cell_path.members.len() > 1) {
        state.cell_path.members.pop();
    }
    state.bottom = false;
}
