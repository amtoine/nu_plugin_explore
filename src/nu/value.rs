//! TODO: documentation
use nu_protocol::{
    ast::{CellPath, PathMember},
    Record, Span, Value,
};

/// TODO: documentation
pub(crate) fn mutate_value_cell(value: &Value, cell_path: &CellPath, cell: &Value) -> Value {
    if cell_path.members.is_empty() {
        return cell.clone();
    }

    if value
        .clone()
        .follow_cell_path(&cell_path.members, false)
        .is_err()
    {
        return value.clone();
    }

    let mut cell_path = cell_path.clone();

    // NOTE: cell_path.members cannot be empty thanks to the guard above
    let first = cell_path.members.first().unwrap();

    match value {
        Value::List { vals, .. } => {
            let id = match first {
                PathMember::Int { val, .. } => *val,
                _ => panic!("first cell path element should be an int"),
            };
            cell_path.members.remove(0);

            let mut vals = vals.clone();
            vals[id] = mutate_value_cell(&vals[id], &cell_path, cell);

            Value::list(vals, Span::unknown())
        }
        Value::Record { val: rec, .. } => {
            let col = match first {
                PathMember::String { val, .. } => val.clone(),
                _ => panic!("first cell path element should be an string"),
            };
            cell_path.members.remove(0);

            let id = rec.cols.iter().position(|x| *x == col).unwrap_or(0);

            let mut vals = rec.vals.clone();
            vals[id] = mutate_value_cell(&vals[id], &cell_path, cell);

            let mut record = Record::new();
            rec.cols.iter().zip(vals).for_each(|(col, val)| {
                record.push(col, val);
            });

            Value::record(record, Span::unknown())
        }
        _ => cell.clone(),
    }
}

/// TODO: documentation
pub(crate) fn is_table(value: &Value) -> bool {
    match value {
        Value::List { vals, .. } => {
            if vals.is_empty() {
                false
            } else {
                match vals[0] {
                    Value::Record { .. } => {
                        let first = vals[0].get_type().to_string();
                        vals.iter().all(|v| v.get_type().to_string() == first)
                    }
                    _ => false,
                }
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{is_table, mutate_value_cell};
    use crate::nu::cell_path::{to_path_member_vec, PM};
    use nu_protocol::{ast::CellPath, record, Value};

    #[test]
    fn value_mutation() {
        let list = Value::test_list(vec![
            Value::test_int(1),
            Value::test_int(2),
            Value::test_int(3),
        ]);
        let record = Value::test_record(record! {
            "a" => Value::test_int(1),
            "b" => Value::test_int(2),
            "c" => Value::test_int(3),
        });

        let cases = vec![
            // simple value -> simple value
            (
                Value::test_string("foo"),
                vec![],
                Value::test_string("bar"),
                Value::test_string("bar"),
            ),
            // list -> simple value
            (
                list.clone(),
                vec![],
                Value::test_nothing(),
                Value::test_nothing(),
            ),
            // record -> simple value
            (
                record.clone(),
                vec![],
                Value::test_nothing(),
                Value::test_nothing(),
            ),
            // mutate a list element with simple value
            (
                list.clone(),
                vec![PM::I(0)],
                Value::test_int(0),
                Value::test_list(vec![
                    Value::test_int(0),
                    Value::test_int(2),
                    Value::test_int(3),
                ]),
            ),
            // mutate a list element with complex value
            (
                list.clone(),
                vec![PM::I(1)],
                record.clone(),
                Value::test_list(vec![Value::test_int(1), record.clone(), Value::test_int(3)]),
            ),
            // invalid list index -> do not mutate
            (
                list.clone(),
                vec![PM::I(5)],
                Value::test_int(0),
                list.clone(),
            ),
            // mutate a record field with a simple value
            (
                record.clone(),
                vec![PM::S("a")],
                Value::test_nothing(),
                Value::test_record(record! {
                    "a" => Value::test_nothing(),
                    "b" => Value::test_int(2),
                    "c" => Value::test_int(3),
                }),
            ),
            // mutate a record field with a complex value
            (
                record.clone(),
                vec![PM::S("c")],
                list.clone(),
                Value::test_record(record! {
                    "a" => Value::test_int(1),
                    "b" => Value::test_int(2),
                    "c" => list.clone(),
                }),
            ),
            // mutate a deeply-nested list element
            (
                Value::test_list(vec![Value::test_list(vec![Value::test_list(vec![
                    Value::test_string("foo"),
                ])])]),
                vec![PM::I(0), PM::I(0), PM::I(0)],
                Value::test_string("bar"),
                Value::test_list(vec![Value::test_list(vec![Value::test_list(vec![
                    Value::test_string("bar"),
                ])])]),
            ),
            // mutate a deeply-nested record field
            (
                Value::test_record(record! {
                    "a" => Value::test_record(record! {
                        "b" => Value::test_record(record! {
                            "c" => Value::test_string("foo"),
                        }),
                    }),
                }),
                vec![PM::S("a"), PM::S("b"), PM::S("c")],
                Value::test_string("bar"),
                Value::test_record(record! {
                    "a" => Value::test_record(record! {
                        "b" => Value::test_record(record! {
                            "c" => Value::test_string("bar"),
                        }),
                    }),
                }),
            ),
        ];

        for (value, members, cell, expected) in cases {
            let cell_path = CellPath {
                members: to_path_member_vec(members),
            };

            // TODO: add proper error messages
            assert_eq!(mutate_value_cell(&value, &cell_path, &cell), expected);
        }
    }

    #[test]
    fn is_a_table() {
        #[rustfmt::skip]
        let table = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("a"),
                "b" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("a"),
                "b" => Value::test_int(1),
            }),
        ]);
        assert_eq!(is_table(&table), true);

        #[rustfmt::skip]
        let not_a_table = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("a"),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("a"),
                "b" => Value::test_int(1),
            }),
        ]);
        assert_eq!(is_table(&not_a_table), false);

        assert_eq!(is_table(&Value::test_int(0)), false);
    }
}
