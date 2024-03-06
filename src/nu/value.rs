use std::collections::HashMap;

use nu_protocol::{
    ast::{CellPath, PathMember},
    record, Record, Span, Type, Value,
};

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

            let cols = rec.columns().cloned().collect();
            let vals = rec
                .values()
                .cloned()
                .enumerate()
                .map(|(i, v)| {
                    if i == id {
                        mutate_value_cell(&v, &cell_path, cell)
                    } else {
                        v
                    }
                })
                .collect();

            Value::record(
                Record::from_raw_cols_vals(cols, vals, Span::unknown(), Span::unknown()).unwrap(),
                Span::unknown(),
            )
        }
        _ => cell.clone(),
    }
}

pub(crate) fn is_table(value: &Value) -> bool {
    match value {
        Value::List { vals, .. } => {
            if vals.is_empty() {
                return false;
            }

            // extract the columns of each row as hashmaps for easier access
            let mut rows = Vec::new();
            for val in vals {
                match val.get_type() {
                    Type::Record(fields) => {
                        rows.push(fields.into_iter().collect::<HashMap<String, Type>>())
                    }
                    _ => return false,
                };
            }

            // check the number of columns for each row
            let n = rows[0].keys().len();
            for row in rows.iter().skip(1) {
                if row.keys().len() != n {
                    return false;
                }
            }

            // check the actual types for each column
            // - if a row has a null, it doesn't count as "not a table"
            // - if two rows are numeric, then the check can continue
            for (key, val) in rows[0].iter() {
                let mut ty = val;

                for row in rows.iter().skip(1) {
                    match row.get(key) {
                        Some(v) => match ty {
                            Type::Nothing => ty = v,
                            _ => {
                                if !matches!(v, Type::Nothing) {
                                    if v.is_numeric() && ty.is_numeric() {
                                    } else if (!v.is_numeric() && ty.is_numeric())
                                        | (v.is_numeric() && !ty.is_numeric())
                                        // NOTE: this might need a bit more work to include more
                                        // tables
                                        | (v != ty)
                                    {
                                        return false;
                                    }
                                }
                            }
                        },
                        None => return false,
                    }
                }
            }

            true
        }
        _ => false,
    }
}

/// this effectively implements the following idempotent `transpose` command written in Nushell
/// ```nushell
/// alias "core transpose" = transpose
///
/// def transpose []: [table -> any, record -> table] {
///     let data = $in
///
///     if ($data | columns) == (seq 1 ($data | columns | length) | into string) {
///         if ($data | columns | length) == 2 {
///             return ($data | core transpose --header-row | into record)
///         } else {
///             return ($data | core transpose --header-row)
///         }
///     }
///
///     $data | core transpose | rename --block {
///         ($in | str replace "column" "" | into int) + 1 | into string
///     }
/// }
///
/// #[test]
/// def transposition [] {
///     use std assert
///
///     assert equal (ls | transpose explore | transpose) (ls)
///     assert equal (open Cargo.toml | transpose | transpose) (open Cargo.toml)
/// }
/// ```
pub(crate) fn transpose(value: &Value) -> Value {
    if is_table(value) {
        let value_rows = match value {
            Value::List { vals, .. } => vals,
            _ => return value.clone(),
        };

        let first_row = value_rows[0].as_record().unwrap();

        let full_columns = (1..=(first_row.len()))
            .map(|i| format!("{i}"))
            .collect::<Vec<String>>();

        if first_row.cols == full_columns {
            if first_row.len() == 2 {
                let cols: Vec<String> = value_rows
                    .iter()
                    .map(|row| row.get_data_by_key("1").unwrap().as_str().unwrap().into())
                    .collect();

                let vals: Vec<Value> = value_rows
                    .iter()
                    .map(|row| row.get_data_by_key("2").unwrap())
                    .collect();

                return Value::record(
                    Record::from_raw_cols_vals(cols, vals, Span::unknown(), Span::unknown())
                        .unwrap(),
                    Span::unknown(),
                );
            } else {
                let mut rows = vec![];
                let cols: Vec<String> = value_rows
                    .iter()
                    .map(|v| v.get_data_by_key("1").unwrap().as_str().unwrap().into())
                    .collect();

                for i in 0..(first_row.len() - 1) {
                    rows.push(Value::record(
                        Record::from_raw_cols_vals(
                            cols.clone(),
                            value_rows
                                .iter()
                                .map(|v| v.get_data_by_key(&format!("{}", i + 2)).unwrap())
                                .collect(),
                            Span::unknown(),
                            Span::unknown(),
                        )
                        .unwrap(),
                        Span::unknown(),
                    ));
                }

                return Value::list(rows, Span::unknown());
            }
        }

        let mut rows = vec![];
        for col in value_rows[0].columns() {
            let mut cols = vec!["1".into()];
            let mut vs = vec![Value::string(col, Span::unknown())];

            for (i, v) in value_rows.iter().enumerate() {
                cols.push(format!("{}", i + 2));
                vs.push(v.get_data_by_key(col).unwrap());
            }

            rows.push(Value::record(
                Record::from_raw_cols_vals(cols, vs, Span::unknown(), Span::unknown()).unwrap(),
                Span::unknown(),
            ));
        }

        return Value::list(rows, Span::unknown());
    }

    match value {
        Value::Record { val: rec, .. } => {
            let mut rows = vec![];
            for (col, val) in rec.iter() {
                rows.push(Value::record(
                    record! {
                        "1" => Value::string(col, Span::unknown()),
                        "2" => val.clone(),
                    },
                    Span::unknown(),
                ));
            }

            Value::list(rows, Span::unknown())
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_table, mutate_value_cell};
    use crate::nu::{
        cell_path::{to_path_member_vec, PM},
        value::transpose,
    };
    use nu_protocol::{ast::CellPath, record, Config, Value};

    fn default_value_repr(value: &Value) -> String {
        value.to_expanded_string(" ", &Config::default())
    }

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
                members: to_path_member_vec(&members),
            };

            let result = mutate_value_cell(&value, &cell_path, &cell);
            assert_eq!(
                result,
                expected,
                "mutating {} at {:?} with {} should give {}, found {}",
                default_value_repr(&value),
                PM::as_cell_path(&members),
                default_value_repr(&cell),
                default_value_repr(&expected),
                default_value_repr(&result)
            );
        }
    }

    #[test]
    fn is_a_table() {
        let table = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("foo"),
                "b" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("bar"),
                "b" => Value::test_int(2),
            }),
        ]);
        assert!(
            is_table(&table),
            "{} should be a table",
            default_value_repr(&table)
        );

        let table_with_out_of_order_columns = Value::test_list(vec![
            Value::test_record(record! {
                "b" => Value::test_int(1),
                "a" => Value::test_string("foo"),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("bar"),
                "b" => Value::test_int(2),
            }),
        ]);
        assert!(
            is_table(&table_with_out_of_order_columns),
            "{} should be a table",
            default_value_repr(&table_with_out_of_order_columns)
        );

        let table_with_nulls = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_nothing(),
                "b" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("bar"),
                "b" => Value::test_int(2),
            }),
        ]);
        assert!(
            is_table(&table_with_nulls),
            "{} should be a table",
            default_value_repr(&table_with_nulls)
        );

        let table_with_number_colum = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("foo"),
                "b" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("bar"),
                "b" => Value::test_float(2.34),
            }),
        ]);
        assert!(
            is_table(&table_with_number_colum),
            "{} should be a table",
            default_value_repr(&table_with_number_colum)
        );

        let not_a_table_missing_field = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("a"),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("a"),
                "b" => Value::test_int(1),
            }),
        ]);
        assert!(
            !is_table(&not_a_table_missing_field),
            "{} should not be a table",
            default_value_repr(&not_a_table_missing_field)
        );

        let not_a_table_incompatible_types = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("a"),
                "b" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("a"),
                "b" => Value::test_list(vec![Value::test_int(1)]),
            }),
        ]);
        assert!(
            !is_table(&not_a_table_incompatible_types),
            "{} should not be a table",
            default_value_repr(&not_a_table_incompatible_types)
        );

        assert!(!is_table(&Value::test_int(0)));
    }

    #[test]
    fn transposition() {
        let record = Value::test_record(record! {
            "a" => Value::test_int(1),
            "b" => Value::test_int(2),
        });
        let expected = Value::test_list(vec![
            Value::test_record(record! {
                "1" => Value::test_string("a"),
                "2" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "1" => Value::test_string("b"),
                "2" => Value::test_int(2),
            }),
        ]);
        let result = transpose(&record);
        assert_eq!(
            result,
            expected,
            "transposing {} should give {}, found {}",
            default_value_repr(&record),
            default_value_repr(&expected),
            default_value_repr(&result)
        );
        // make sure `transpose` is an *involution*
        let result = transpose(&expected);
        assert_eq!(
            result,
            record,
            "transposing {} should give {}, found {}",
            default_value_repr(&expected),
            default_value_repr(&record),
            default_value_repr(&result)
        );

        let table = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_int(1),
                "b" => Value::test_int(2),
            }),
            Value::test_record(record! {
                "a" => Value::test_int(3),
                "b" => Value::test_int(4),
            }),
        ]);
        let expected = Value::test_list(vec![
            Value::test_record(record! {
                "1" => Value::test_string("a"),
                "2" => Value::test_int(1),
                "3" => Value::test_int(3),
            }),
            Value::test_record(record! {
                "1" => Value::test_string("b"),
                "2" => Value::test_int(2),
                "3" => Value::test_int(4),
            }),
        ]);
        let result = transpose(&table);
        assert_eq!(
            result,
            expected,
            "transposing {} should give {}, found {}",
            default_value_repr(&table),
            default_value_repr(&expected),
            default_value_repr(&result)
        );
        // make sure `transpose` is an *involution*
        let result = transpose(&expected);
        assert_eq!(
            result,
            table,
            "transposing {} should give {}, found {}",
            default_value_repr(&expected),
            default_value_repr(&table),
            default_value_repr(&result)
        );

        assert_eq!(
            transpose(&Value::test_string("foo")),
            Value::test_string("foo")
        );

        assert_eq!(
            transpose(&Value::test_list(vec![
                Value::test_int(1),
                Value::test_int(2)
            ])),
            Value::test_list(vec![Value::test_int(1), Value::test_int(2)])
        );
    }
}
