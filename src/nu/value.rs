use std::collections::HashMap;

use nu_protocol::{
    ast::{CellPath, PathMember},
    record, Record, Span, Type, Value,
};

#[derive(Debug, PartialEq)]
pub(crate) enum Table {
    /// value is a list but with no items in it
    Empty,
    /// row at index {0} should be a record but is a {1}
    RowNotARecord(usize, Type),
    /// row at index {0} should have same length as first row, {2}, but has
    /// length {1}
    RowIncompatibleLen(usize, usize, usize),
    /// row at index {0} and subkey {1} has type {2} but was expected to have
    /// the same type as first row {3}
    RowIncompatibleType(usize, String, Type, Type),
    /// row at index {0} contains an invalid key {1} but expected one of {2}
    RowInvalidKey(usize, String, Vec<String>),
    /// value is a valid table
    IsValid,
    /// valis is not even a list
    NotAList,
}

impl Table {
    pub(crate) fn to_msg(&self) -> Option<String> {
        match self {
            Table::Empty => None,
            Table::RowNotARecord(i, t) => Some(format!("row $.{} is not a record: {}", i, t)),
            Table::RowIncompatibleLen(i, l, e) => Some(format!(
                "row $.{} has incompatible length with first row: expected {} found {}",
                i, e, l
            )),
            Table::RowIncompatibleType(i, k, t, e) => Some(format!(
                "cell $.{}.{} has incompatible type with first row: expected {} found {}",
                i, k, e, t
            )),
            Table::RowInvalidKey(i, k, ks) => Some(format!(
                "row $.{} does not contain key '{}': list of keys {:?}",
                i, k, ks
            )),
            Table::NotAList => None,
            Table::IsValid => None,
        }
    }
}

/// mutate the input `value`, changing the _value_ at `cell_path` into the `cell` argument
///
/// > **Note**  
/// > returns [`None`] if the `cell_path` is not valid in `value`.
pub(crate) fn mutate_value_cell(
    value: &Value,
    cell_path: &CellPath,
    cell: &Value,
) -> Option<Value> {
    if cell_path.members.is_empty() {
        return Some(cell.clone());
    }

    if value
        .clone()
        .follow_cell_path(&cell_path.members, false)
        .is_err()
    {
        return None;
    }

    let mut cell_path = cell_path.clone();

    // NOTE: `cell_path.members` cannot be empty because the last branch of the match bellot
    // does not call `aux` recursively
    let first = cell_path.members.first().unwrap();

    let res = match value {
        Value::List { vals, .. } => {
            let id = match first {
                PathMember::Int { val, .. } => *val,
                _ => unreachable!(),
            };
            cell_path.members.remove(0);

            let mut vals = vals.clone();
            vals[id] = mutate_value_cell(&vals[id], &cell_path, cell).unwrap();

            Value::list(vals, Span::unknown())
        }
        Value::Record { val: rec, .. } => {
            let col = match first {
                PathMember::String { val, .. } => val.clone(),
                _ => unreachable!(),
            };
            cell_path.members.remove(0);

            let id = rec.columns().position(|x| *x == col).unwrap_or(0);

            let cols = rec.columns().cloned().collect();
            let vals = rec
                .values()
                .cloned()
                .enumerate()
                .map(|(i, v)| {
                    if i == id {
                        mutate_value_cell(&v, &cell_path, cell).unwrap()
                    } else {
                        v
                    }
                })
                .collect();

            Value::record(
                // NOTE: this cannot fail because `cols` and `vals` have the same length by
                // contruction, they have been constructed by iterating over `rec.columns()`
                // and `rec.values()` respectively, both of which have the same length
                Record::from_raw_cols_vals(cols, vals, Span::unknown(), Span::unknown()).unwrap(),
                Span::unknown(),
            )
        }
        _ => cell.clone(),
    };

    Some(res)
}

pub(crate) fn is_table(value: &Value, loose: bool) -> Table {
    match value {
        Value::List { vals, .. } => {
            if vals.is_empty() {
                return Table::Empty;
            }

            // extract the columns of each row as hashmaps for easier access
            let mut rows = Vec::new();
            for (i, val) in vals.iter().enumerate() {
                match val.get_type() {
                    Type::Record(fields) => rows.push(
                        Vec::from(fields)
                            .into_iter()
                            .collect::<HashMap<String, Type>>(),
                    ),
                    t => return Table::RowNotARecord(i, t),
                };
            }

            if loose {
                return Table::IsValid;
            }

            // check the number of columns for each row
            let n = rows[0].keys().len();
            for (i, row) in rows.iter().skip(1).enumerate() {
                if row.keys().len() != n {
                    return Table::RowIncompatibleLen(i + 1, row.keys().len(), n);
                }
            }

            // check the actual types for each column
            // - if a row has a null, it doesn't count as "not a table"
            // - if two rows are numeric, then the check can continue
            for (key, val) in rows[0].iter() {
                let mut ty = val;

                for (i, row) in rows.iter().skip(1).enumerate() {
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
                                        return Table::RowIncompatibleType(
                                            i + 1,
                                            key.clone(),
                                            v.clone(),
                                            ty.clone(),
                                        );
                                    }
                                }
                            }
                        },
                        None => {
                            let mut keys = row.keys().cloned().collect::<Vec<String>>();
                            keys.sort();
                            return Table::RowInvalidKey(i + 1, key.clone(), keys);
                        }
                    }
                }
            }

            Table::IsValid
        }
        _ => Table::NotAList,
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
// WARNING: some _unwraps_ haven't been proven to be safe in this function
pub(crate) fn transpose(value: &Value) -> Value {
    if matches!(is_table(value, false), Table::IsValid) {
        let value_rows = match value {
            Value::List { vals, .. } => vals,
            _ => return value.clone(),
        };

        // NOTE: because `value` is a valid table, it's first row is guaranteed to be a record
        let first_row = value_rows[0].as_record().unwrap();

        let full_columns = (1..=(first_row.len()))
            .map(|i| format!("{i}"))
            .collect::<Vec<String>>();

        if first_row.columns().cloned().collect::<Vec<_>>() == full_columns {
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
                    // NOTE: `cols` and `value_rows` have the same length by contruction
                    // because they have been created by iterating over `value_rows`
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
                        // NOTE: `cols` and `value_rows` have the same length by contruction
                        // because `cols` has been created by iterating over `value_rows`
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
                // NOTE: `cols` and `vs` have the same length by construction
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
        value::{transpose, Table},
    };
    use nu_protocol::{ast::CellPath, record, Config, Type, Value};

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
                Some(Value::test_string("bar")),
            ),
            // list -> simple value
            (
                list.clone(),
                vec![],
                Value::test_nothing(),
                Some(Value::test_nothing()),
            ),
            // record -> simple value
            (
                record.clone(),
                vec![],
                Value::test_nothing(),
                Some(Value::test_nothing()),
            ),
            // mutate a list element with simple value
            (
                list.clone(),
                vec![PM::I(0)],
                Value::test_int(0),
                Some(Value::test_list(vec![
                    Value::test_int(0),
                    Value::test_int(2),
                    Value::test_int(3),
                ])),
            ),
            // mutate a list element with complex value
            (
                list.clone(),
                vec![PM::I(1)],
                record.clone(),
                Some(Value::test_list(vec![
                    Value::test_int(1),
                    record.clone(),
                    Value::test_int(3),
                ])),
            ),
            // invalid list index
            (list.clone(), vec![PM::I(5)], Value::test_int(0), None),
            // mutate a record field with a simple value
            (
                record.clone(),
                vec![PM::S("a")],
                Value::test_nothing(),
                Some(Value::test_record(record! {
                    "a" => Value::test_nothing(),
                    "b" => Value::test_int(2),
                    "c" => Value::test_int(3),
                })),
            ),
            // mutate a record field with a complex value
            (
                record.clone(),
                vec![PM::S("c")],
                list.clone(),
                Some(Value::test_record(record! {
                    "a" => Value::test_int(1),
                    "b" => Value::test_int(2),
                    "c" => list.clone(),
                })),
            ),
            // mutate a deeply-nested list element
            (
                Value::test_list(vec![Value::test_list(vec![Value::test_list(vec![
                    Value::test_string("foo"),
                ])])]),
                vec![PM::I(0), PM::I(0), PM::I(0)],
                Value::test_string("bar"),
                Some(Value::test_list(vec![Value::test_list(vec![
                    Value::test_list(vec![Value::test_string("bar")]),
                ])])),
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
                Some(Value::test_record(record! {
                    "a" => Value::test_record(record! {
                        "b" => Value::test_record(record! {
                            "c" => Value::test_string("bar"),
                        }),
                    }),
                })),
            ),
            // try to mutate bad cell path
            (
                Value::test_record(record! {
                    "a" => Value::test_record(record! {
                        "b" => Value::test_record(record! {
                            "c" => Value::test_string("foo"),
                        }),
                    }),
                }),
                vec![PM::S("a"), PM::I(0), PM::S("c")],
                Value::test_string("bar"),
                None,
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
                default_value_repr(&match expected.clone() {
                    Some(v) => v,
                    None => Value::test_nothing(),
                }),
                default_value_repr(&match result.clone() {
                    Some(v) => v,
                    None => Value::test_nothing(),
                }),
            );
        }
    }

    #[test]
    fn is_a_table() {
        let simple_table = Value::test_list(vec![
            Value::test_record(record! {
                "a" => Value::test_string("foo"),
                "b" => Value::test_int(1),
            }),
            Value::test_record(record! {
                "a" => Value::test_string("bar"),
                "b" => Value::test_int(2),
            }),
        ]);
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
        for table in [
            simple_table,
            table_with_out_of_order_columns,
            table_with_nulls,
            table_with_number_colum,
        ] {
            assert_eq!(
                is_table(&table, false),
                Table::IsValid,
                "{} should be a table",
                default_value_repr(&table)
            );
        }

        let not_a_table_missing_field = (
            Value::test_list(vec![
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                }),
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                    "b" => Value::test_int(1),
                }),
            ]),
            Table::RowIncompatibleLen(1, 2, 1),
        );
        let not_a_table_incompatible_types = (
            Value::test_list(vec![
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                    "b" => Value::test_int(1),
                }),
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                    "b" => Value::test_list(vec![Value::test_int(1)]),
                }),
            ]),
            Table::RowIncompatibleType(
                1,
                "b".to_string(),
                Type::List(Box::new(Type::Int)),
                Type::Int,
            ),
        );
        let not_a_table_row_not_record = (
            Value::test_list(vec![
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                    "b" => Value::test_int(1),
                }),
                Value::test_int(0),
            ]),
            Table::RowNotARecord(1, Type::Int),
        );
        let not_a_table_row_invalid_key = (
            Value::test_list(vec![
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                    "b" => Value::test_int(1),
                }),
                Value::test_record(record! {
                    "a" => Value::test_string("a"),
                    "c" => Value::test_int(2),
                }),
            ]),
            Table::RowInvalidKey(1, "b".into(), vec!["a".into(), "c".into()]),
        );
        for (not_a_table, expected) in [
            not_a_table_missing_field,
            not_a_table_incompatible_types,
            not_a_table_row_not_record,
            not_a_table_row_invalid_key,
        ] {
            assert_eq!(
                is_table(&not_a_table, false),
                expected,
                "{} should not be a table",
                default_value_repr(&not_a_table)
            );
        }

        assert_eq!(is_table(&Value::test_int(0), false), Table::NotAList);
        assert_eq!(is_table(&Value::test_list(vec![]), false), Table::Empty);
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
