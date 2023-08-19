//! navigate in the data in all directions
use nu_protocol::{ast::PathMember, Span, Value};

use super::app::State;

/// specify a vertical direction in which to go in the data
pub(super) enum Direction {
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
pub(super) fn go_up_or_down_in_data(state: &mut State, input: &Value, direction: Direction) {
    if state.bottom {
        return;
    }

    let direction = match direction {
        Direction::Up => -1,
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
                        let index = cols.iter().position(|x| x == &val).unwrap() as i32;
                        let len = cols.len() as i32;
                        let new_index = (index + direction + len) % len;

                        cols[new_index as usize].clone()
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

/// go one level deeper in the data
///
/// > :bulb: **Note**  
/// > this function will
/// > - push a new *cell path* member to the state if there is more depth ahead
/// > - mark the state as *at the bottom* if the value at the new depth is of a simple type
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

/// pop one level of depth from the data
///
/// > :bulb: **Note**
/// > - the state is always marked as *not at the bottom*
/// > - the state *cell path* can have it's last member popped if possible
pub(super) fn go_back_in_data(state: &mut State) {
    if !state.bottom & (state.cell_path.members.len() > 1) {
        state.cell_path.members.pop();
    }
    state.bottom = false;
}
