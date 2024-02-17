//! the module responsible for rendering the TUI
use crate::nu::{strings::SpecialString, value::is_table};

use super::config::{repr_keycode, Layout};
use super::{App, Config, Mode};
use crossterm::event::KeyCode;
use nu_protocol::ast::PathMember;
use nu_protocol::{Record, Type, Value};
use ratatui::prelude::Backend;
use ratatui::{
    prelude::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap,
    },
    Frame,
};

/// render the whole ui
pub(super) fn render_ui<B: Backend>(
    frame: &mut Frame<'_, B>,
    app: &App,
    config: &Config,
    error: Option<&str>,
) {
    render_data(frame, app, config);
    if config.show_cell_path {
        render_cell_path(frame, app);
    }

    match error {
        Some(err) => render_error(frame, err),
        None => {
            render_status_bar(frame, app, config);

            if app.mode == Mode::Insert {
                app.editor.render(frame, config);
            }
        }
    }
}

pub(super) fn render_error<B: Backend>(frame: &mut Frame<'_, B>, error: &str) {
    let bottom_two_lines = Rect::new(0, frame.size().height - 2, frame.size().width, 2);

    let lines = vec![
        Line::from(Span::styled(
            format!("Err: {error}"),
            Style::default().fg(Color::Red),
        )),
        Line::from(Span::styled(
            "Press any key to continue exploring the data.",
            Style::default().fg(Color::Blue),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Left),
        bottom_two_lines,
    );
}

/// a common representation for an explore row
#[derive(Clone, Debug, PartialEq)]
struct DataRowRepr {
    name: Option<String>,
    shape: String,
    data: String,
}

impl DataRowRepr {
    #[cfg(test)]
    fn unnamed(data: impl Into<String>, shape: impl Into<String>) -> Self {
        Self {
            name: None,
            shape: shape.into(),
            data: data.into(),
        }
    }

    #[cfg(test)]
    fn named(name: impl Into<String>, data: impl Into<String>, shape: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            shape: shape.into(),
            data: data.into(),
        }
    }
}

/// compute the preview representation of a list
///
/// > see the tests for detailed examples
fn repr_list(vals: &[Value]) -> DataRowRepr {
    let data = match vals.len() {
        0 => "[]".into(),
        1 => "[1 item]".into(),
        x => format!("[{} items]", x),
    };

    DataRowRepr {
        name: None,
        shape: "list".into(),
        data,
    }
}

/// compute the preview representation of a record
///
/// > see the tests for detailed examples
fn repr_record(cols: &[String]) -> DataRowRepr {
    let data = match cols.len() {
        0 => "{}".into(),
        1 => "{1 field}".into(),
        x => format!("{{{} fields}}", x),
    };

    DataRowRepr {
        name: None,
        shape: "record".into(),
        data,
    }
}

/// compute the preview representation of a simple value
///
/// > see the tests for detailed examples
fn repr_simple_value(value: &Value) -> DataRowRepr {
    let shape = match value {
        Value::String { val, .. } => match SpecialString::parse(val) {
            Some(x) => x.to_string(),
            None => value.get_type().to_string(),
        },
        x => x.get_type().to_string(),
    };
    DataRowRepr {
        name: None,
        shape,
        // FIXME: use a real config
        data: value.into_string(" ", &nu_protocol::Config::default()),
    }
}

/// compute the preview representation of a value
///
/// > see the tests for detailed examples
fn repr_value(value: &Value) -> DataRowRepr {
    match value {
        Value::List { vals, .. } => repr_list(vals),
        Value::Record { val: rec, .. } => repr_record(&rec.cols),
        x => repr_simple_value(x),
    }
}

/// compute the row / item representation of a complete Nushell Value
///
/// > see the tests for detailed examples
fn repr_data(data: &Value) -> Vec<DataRowRepr> {
    match data {
        Value::List { vals, .. } => {
            if vals.is_empty() {
                vec![DataRowRepr {
                    name: None,
                    shape: "list".into(),
                    data: "[]".into(),
                }]
            } else {
                vals.iter().map(repr_value).collect::<Vec<DataRowRepr>>()
            }
        }
        Value::Record { val: rec, .. } => {
            if rec.cols.is_empty() {
                vec![DataRowRepr {
                    name: None,
                    shape: "record".into(),
                    data: "{}".into(),
                }]
            } else {
                rec.iter()
                    .map(|(col, val)| {
                        let mut repr = repr_value(val);
                        repr.name = Some(col.to_string());
                        repr
                    })
                    .collect::<Vec<DataRowRepr>>()
            }
        }
        value => vec![repr_simple_value(value)],
    }
}

/// compute the representation of a complete Nushell table
///
/// > see the tests for detailed examples
fn repr_table(table: &[Record]) -> (Vec<String>, Vec<String>, Vec<Vec<String>>) {
    let mut shapes = vec![Type::Nothing; table[0].len()];

    let mut rows = vec![vec![]; table.len()];

    for (i, row) in table.iter().enumerate() {
        for (j, col) in table[0].cols.iter().enumerate() {
            // NOTE: because `table` is a valid table, this should always be a `Some`
            let val = row.get(col).unwrap();

            let cell_type = val.get_type();
            if !matches!(cell_type, Type::Nothing) {
                if shapes[j].is_numeric() && cell_type.is_numeric() && (shapes[j] != cell_type) {
                    shapes[j] = Type::Number;
                } else {
                    shapes[j] = cell_type;
                }
            }

            rows[i].push(repr_value(val).data);
        }
    }

    (
        table[0].cols.clone(),
        shapes.iter().map(|s| s.to_string()).collect(),
        rows,
    )
}

/// render the whole data
///
/// the layout can be changed from [`crate::config::Config::layout`].
///
/// the data will be rendered on top of the bar, and on top of the cell path in case
/// [`crate::config::Config::show_cell_path`] is set to `true`.
fn render_data<B: Backend>(frame: &mut Frame<'_, B>, app: &App, config: &Config) {
    let mut data_path = app.position.members.clone();
    let current = if !app.is_at_bottom() {
        data_path.pop()
    } else {
        None
    };

    let value = app
        .value
        .clone()
        .follow_cell_path(&data_path, false)
        .unwrap_or_else(|_| {
            panic!(
                "unexpected error when following {:?} in {}",
                app.position.members,
                app.value.into_string(" ", &nu_protocol::Config::default())
            )
        });

    let table_type = is_table(&value);
    let is_a_table = matches!(table_type, crate::nu::value::Table::IsValid);

    let mut data_frame_height = if config.show_cell_path {
        frame.size().height - 2
    } else {
        frame.size().height - 1
    };
    if !is_a_table {
        let msg = match table_type {
            crate::nu::value::Table::Empty => None,
            crate::nu::value::Table::RowNotARecord(i, t) => {
                Some(format!("row $.{} is not a record: {}", i, t))
            }
            crate::nu::value::Table::RowIncompatibleLen(i, l, e) => Some(format!(
                "row $.{} has incompatible length with first row: expected {} found {}",
                i, e, l
            )),
            crate::nu::value::Table::RowIncompatibleType(i, k, t, e) => Some(format!(
                "cell $.{}.{} has incompatible type with first row: expected {} found {}",
                i, k, e, t
            )),
            crate::nu::value::Table::RowInvalidKey(i, k, ks) => Some(format!(
                "row $.{} does not contain key '{}': list of keys {:?}",
                i, k, ks
            )),
            crate::nu::value::Table::NotAList => None,
            crate::nu::value::Table::IsValid => unreachable!(),
        };

        if let Some(msg) = msg {
            data_frame_height -= 1;
            frame.render_widget(
                Paragraph::new(msg).alignment(Alignment::Right).style(
                    Style::default()
                        .bg(config.colors.warning.background)
                        .fg(config.colors.warning.foreground),
                ),
                Rect::new(0, data_frame_height, frame.size().width, 1),
            );
        }
    }
    let rect_without_bottom_bar = Rect::new(0, 0, frame.size().width, data_frame_height);

    let normal_name_style = Style::default()
        .fg(config.colors.normal.name.foreground)
        .bg(config.colors.normal.name.background);
    let normal_data_style = Style::default()
        .fg(config.colors.normal.data.foreground)
        .bg(config.colors.normal.data.background);
    let normal_shape_style = Style::default()
        .fg(config.colors.normal.shape.foreground)
        .bg(config.colors.normal.shape.background);
    let highlight_style = Style::default()
        .fg(config.colors.selected.foreground)
        .bg(config.colors.selected.background)
        .add_modifier(config.colors.selected_modifier);

    let selected = match current {
        Some(PathMember::Int { val, .. }) => val,
        Some(PathMember::String { val, .. }) => {
            value.columns().position(|x| x == &val).unwrap_or(0)
        }
        None => 0,
    };

    if is_a_table {
        let (columns, shapes, cells) = match value {
            Value::List { vals, .. } => {
                let recs = vals
                    .iter()
                    .map(|v| v.as_record().unwrap().clone())
                    .collect::<Vec<Record>>();
                repr_table(&recs)
            }
            _ => panic!("value is a table but is not a list"),
        };

        let header = columns
            .iter()
            .zip(shapes)
            .map(|(c, s)| {
                let spans = vec![
                    Span::styled(c, normal_name_style),
                    " (".into(),
                    Span::styled(s, normal_shape_style),
                    ")".into(),
                ];

                Cell::from(Line::from(spans))
            })
            .collect::<Vec<Cell>>();

        let widths = header
            .iter()
            // FIXME: use an appropriate constraint here
            .map(|_| Constraint::Min(25))
            .collect::<Vec<Constraint>>();

        let header = Row::new(header).height(1);

        let rows: Vec<Row> = cells
            .iter()
            .map(|r| Row::new(r.iter().cloned().map(Cell::from).collect::<Vec<Cell>>()))
            .collect();

        let table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(highlight_style)
            .highlight_symbol(&config.colors.selected_symbol)
            .widths(&widths);

        frame.render_stateful_widget(
            table,
            rect_without_bottom_bar,
            &mut TableState::default().with_selected(Some(selected)),
        );

        return;
    }

    match config.layout {
        Layout::Compact => {
            let items: Vec<ListItem> = repr_data(&value)
                .iter()
                .cloned()
                .map(|row| {
                    let mut spans = vec![];
                    if let Some(name) = row.name {
                        spans.push(Span::styled(name, normal_name_style));
                        spans.push(": ".into());
                    }
                    spans.push("(".into());
                    spans.push(Span::styled(row.shape, normal_shape_style));
                    spans.push(") ".into());
                    spans.push(Span::styled(row.data, normal_data_style));

                    ListItem::new(Line::from(spans))
                })
                .collect();

            let items = List::new(items)
                .highlight_style(highlight_style)
                .highlight_symbol(&config.colors.selected_symbol);

            frame.render_stateful_widget(
                items,
                rect_without_bottom_bar,
                &mut ListState::default().with_selected(Some(selected)),
            )
        }
        Layout::Table => {
            let (header, rows, constraints) = match value {
                Value::List { .. } => {
                    let header = Row::new(vec![
                        Cell::from("item")
                            .style(normal_data_style.add_modifier(Modifier::REVERSED)),
                        Cell::from("shape")
                            .style(normal_shape_style.add_modifier(Modifier::REVERSED)),
                    ]);
                    let rows: Vec<Row> = repr_data(&value)
                        .iter()
                        .cloned()
                        .map(|row| {
                            let data_style = match row.data.as_str() {
                                "record" | "list" => normal_data_style.add_modifier(Modifier::DIM),
                                _ => normal_data_style,
                            };

                            Row::new(vec![
                                Cell::from(row.data).style(data_style),
                                Cell::from(row.shape).style(normal_shape_style),
                            ])
                        })
                        .collect();

                    let constraints = vec![Constraint::Percentage(90), Constraint::Percentage(10)];

                    (header, rows, constraints)
                }
                Value::Record { .. } => {
                    let header = Row::new(vec![
                        Cell::from("key").style(normal_name_style.add_modifier(Modifier::REVERSED)),
                        Cell::from("field")
                            .style(normal_data_style.add_modifier(Modifier::REVERSED)),
                        Cell::from("shape")
                            .style(normal_shape_style.add_modifier(Modifier::REVERSED)),
                    ]);

                    let rows: Vec<Row> = repr_data(&value)
                        .iter()
                        .cloned()
                        .map(|row| {
                            let data_style = match row.data.as_str() {
                                "record" | "list" => normal_data_style.add_modifier(Modifier::DIM),
                                _ => normal_data_style,
                            };

                            Row::new(vec![
                                Cell::from(row.name.unwrap_or("".into())).style(normal_name_style),
                                Cell::from(row.data).style(data_style),
                                Cell::from(row.shape).style(normal_shape_style),
                            ])
                        })
                        .collect();

                    let constraints = vec![
                        Constraint::Percentage(20),
                        Constraint::Percentage(70),
                        Constraint::Percentage(10),
                    ];

                    (header, rows, constraints)
                }
                v => {
                    let repr = repr_simple_value(&v);
                    let spans = vec![
                        Span::styled(repr.data, normal_data_style),
                        " is of shape ".into(),
                        Span::styled(repr.shape, normal_shape_style),
                    ];

                    frame.render_widget(
                        Paragraph::new(Line::from(spans))
                            .block(Block::default().borders(Borders::ALL))
                            .wrap(Wrap { trim: false }),
                        rect_without_bottom_bar,
                    );
                    return;
                }
            };

            let table = if config.show_table_header {
                Table::new(rows).header(header.height(1))
            } else {
                Table::new(rows)
            }
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(highlight_style)
            .highlight_symbol(&config.colors.selected_symbol)
            .widths(&constraints);

            frame.render_stateful_widget(
                table,
                rect_without_bottom_bar,
                &mut TableState::default().with_selected(Some(selected)),
            )
        }
    }
}

/// render the cell path just above the status bar
///
/// this line can be removed through config, see [`crate::config::Config::show_cell_path`]
///
/// # Examples
/// > :bulb: **Note**  
/// > the `...` are here to signify that the bar might be truncated and the `||` at the start and
/// the end of the lines are just to represent the borders of the terminal but will not appear in
/// the TUI.
/// - at the beginning
/// ```text
/// ||cell path: $.   ...||
/// ```
/// - after some navigation, might look like
/// ```text
/// ||cell path: $.foo.bar.2.baz    ...||
/// ```
fn render_cell_path<B: Backend>(frame: &mut Frame<'_, B>, app: &App) {
    let next_to_bottom_bar_rect = Rect::new(0, frame.size().height - 2, frame.size().width, 1);
    let cell_path = format!(
        "cell path: $.{}",
        app.position
            .members
            .iter()
            .map(|m| {
                match m {
                    PathMember::Int { val, .. } => val.to_string(),
                    PathMember::String { val, .. } => val.to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join(".")
    );

    frame.render_widget(
        Paragraph::new(cell_path).alignment(Alignment::Left),
        next_to_bottom_bar_rect,
    );
}

/// render the status bar at the bottom
///
/// the bar takes the last line of the TUI only and renders, from left to right
/// - the current mode
/// - hints about next bindings to press and actions to do
///
/// the color depending of the mode is completely configurable!
///
/// # Examples
/// > :bulb: **Note**  
/// > - the `...` are here to signify that the bar might be truncated and the `||` at the start and
/// the end of the lines are just to represent the borders of the terminal but will not appear in
/// the TUI.
/// > - these examples use the default bindings
/// - in NORMAL mode
/// ```text
/// ||NORMAL  ...                                     i to INSERT | hjkl to move around | p to peek | q to quit||
/// ```
/// - in INSERT mode
/// ```text
/// ||INSERT  ...                                                                               <esc> to NORMAL||
/// ```
/// - in PEEKING mode
/// ```text
/// ||PEEKING ... <esc> to NORMAL | a to peek all | c to peek current view | u to peek under cursor | q to quit||
/// ```
fn render_status_bar<B: Backend>(frame: &mut Frame<'_, B>, app: &App, config: &Config) {
    let bottom_bar_rect = Rect::new(0, frame.size().height - 1, frame.size().width, 1);

    let bg_style = match app.mode {
        Mode::Normal => Style::default().bg(config.colors.status_bar.normal.background),
        Mode::Insert => Style::default().bg(config.colors.status_bar.insert.background),
        Mode::Peeking => Style::default().bg(config.colors.status_bar.peek.background),
        Mode::Bottom => Style::default().bg(config.colors.status_bar.bottom.background),
    };

    let style = match app.mode {
        Mode::Normal => bg_style.fg(config.colors.status_bar.normal.foreground),
        Mode::Insert => bg_style.fg(config.colors.status_bar.insert.foreground),
        Mode::Peeking => bg_style.fg(config.colors.status_bar.peek.foreground),
        Mode::Bottom => bg_style.fg(config.colors.status_bar.bottom.foreground),
    };

    let hints = match app.mode {
        Mode::Normal => format!(
            "{} to {} | {}{}{}{} to move around | {} to peek | {} to transpose | {} to quit",
            repr_keycode(&config.keybindings.insert),
            Mode::Insert,
            repr_keycode(&config.keybindings.navigation.left),
            repr_keycode(&config.keybindings.navigation.down),
            repr_keycode(&config.keybindings.navigation.up),
            repr_keycode(&config.keybindings.navigation.right),
            repr_keycode(&config.keybindings.peek),
            repr_keycode(&config.keybindings.transpose),
            repr_keycode(&config.keybindings.quit),
        ),
        Mode::Insert => format!(
            "{} to quit | {}{}{}{} to move the cursor | {}{} to delete characters | {} to confirm",
            repr_keycode(&KeyCode::Esc),
            repr_keycode(&KeyCode::Left),
            repr_keycode(&KeyCode::Right),
            repr_keycode(&KeyCode::Up),
            repr_keycode(&KeyCode::Down),
            repr_keycode(&KeyCode::Backspace),
            repr_keycode(&KeyCode::Delete),
            repr_keycode(&KeyCode::Enter),
        ),
        Mode::Peeking => format!(
            "{} to {} | {} to peek all | {} to peek current view | {} to peek under cursor | {} to peek the cell path",
            repr_keycode(&config.keybindings.normal),
            Mode::Normal,
            repr_keycode(&config.keybindings.peeking.all),
            repr_keycode(&config.keybindings.peeking.view),
            repr_keycode(&config.keybindings.peeking.under),
            repr_keycode(&config.keybindings.peeking.cell_path),
        ),
        Mode::Bottom => format!(
            "{} to {} | {} to peek | {} to quit",
            repr_keycode(&config.keybindings.navigation.left),
            Mode::Normal,
            repr_keycode(&config.keybindings.peek),
            repr_keycode(&config.keybindings.quit),
        ),
    };

    let left = Line::from(Span::styled(
        format!(" {} ", app.mode),
        style.add_modifier(Modifier::REVERSED),
    ));
    let right = Line::from(Span::styled(hints, style));

    frame.render_widget(
        Paragraph::new(left)
            .alignment(Alignment::Left)
            .style(bg_style),
        bottom_bar_rect,
    );
    frame.render_widget(
        Paragraph::new(right).alignment(Alignment::Right),
        bottom_bar_rect,
    );
}

// TODO: add proper assert error messages
#[cfg(test)]
mod tests {
    use nu_protocol::{record, Value};

    use super::{repr_data, repr_list, repr_record, repr_simple_value, repr_table, DataRowRepr};

    #[test]
    fn simple_value() {
        #[rustfmt::skip]
        let cases = vec![
            (Value::test_string("foo"), DataRowRepr::unnamed("foo", "string")),
            (Value::test_int(1), DataRowRepr::unnamed("1", "int")),
            (Value::test_bool(true), DataRowRepr::unnamed("true", "bool")),
            (Value::test_nothing(), DataRowRepr::unnamed("", "nothing")),
            (Value::test_string("foo"), DataRowRepr::unnamed("foo", "string")),
        ];

        for (value, expected) in cases {
            assert_eq!(repr_simple_value(&value), expected);
        }
    }

    #[test]
    fn list() {
        let list = vec![
            Value::test_string("a"),
            Value::test_int(1),
            Value::test_bool(false),
        ];

        #[rustfmt::skip]
        let cases = vec![
            (list, DataRowRepr::unnamed("[3 items]", "list")),
            (vec![], DataRowRepr::unnamed("[]", "list")),
            (vec![Value::test_nothing()], DataRowRepr::unnamed("[1 item]", "list")),
        ];

        for (list, expected) in cases {
            assert_eq!(repr_list(&list), expected);
        }
    }

    #[test]
    fn record() {
        #[rustfmt::skip]
        let cases = vec![
            (vec!["a", "b", "c"], DataRowRepr::unnamed("{3 fields}", "record")),
            (vec![], DataRowRepr::unnamed("{}", "record")),
            (vec!["a"], DataRowRepr::unnamed("{1 field}", "record")),
        ];

        for (record, expected) in cases {
            assert_eq!(
                repr_record(
                    &record
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                ),
                expected
            );
        }
    }

    #[ignore = "repr_value is just a direct wrapper around repr_list, repr_record and repr_simple_value"]
    #[test]
    fn value() {}

    #[test]
    fn data() {
        let data = Value::test_record(record! {
            "l" => Value::test_list(vec![
                Value::test_string("my"),
                Value::test_string("list"),
                Value::test_string("elements"),
            ]),
            "r" => Value::test_record(record! {
                "a" => Value::test_int(1),
                "b" => Value::test_int(2),
            }),
            "s" => Value::test_string("some string"),
            "i" => Value::test_int(123),
        });

        let result = repr_data(&data);
        let expected: Vec<DataRowRepr> = vec![
            DataRowRepr::named("l", "[3 items]", "list"),
            DataRowRepr::named("r", "{2 fields}", "record"),
            DataRowRepr::named("s", "some string", "string"),
            DataRowRepr::named("i", "123", "int"),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn repr_simple_table() {
        let table = vec![
            record! {
                "a" => Value::test_string("x"),
                "b" => Value::test_int(1),
            },
            record! {
                "a" => Value::test_string("y"),
                "b" => Value::test_int(2),
            },
        ];

        let expected = (
            vec!["a".into(), "b".into()],
            vec!["string".into(), "int".into()],
            vec![vec!["x".into(), "1".into()], vec!["y".into(), "2".into()]],
        );

        assert_eq!(repr_table(&table), expected);
    }

    #[test]
    fn repr_table_with_empty_column() {
        let table = vec![
            record! {
                "a" => Value::test_nothing(),
                "b" => Value::test_int(1),
            },
            record! {
                "a" => Value::test_nothing(),
                "b" => Value::test_int(2),
            },
        ];

        let expected = (
            vec!["a".into(), "b".into()],
            vec!["nothing".into(), "int".into()],
            vec![vec!["".into(), "1".into()], vec!["".into(), "2".into()]],
        );

        assert_eq!(repr_table(&table), expected);
    }

    #[test]
    fn repr_table_with_shuffled_columns() {
        let table = vec![
            record! {
                "b" => Value::test_int(1),
                "a" => Value::test_string("x"),
            },
            record! {
                "a" => Value::test_string("y"),
                "b" => Value::test_int(2),
            },
        ];

        let expected = (
            vec!["b".into(), "a".into()],
            vec!["int".into(), "string".into()],
            vec![vec!["1".into(), "x".into()], vec!["2".into(), "y".into()]],
        );

        assert_eq!(repr_table(&table), expected);
    }

    #[test]
    fn repr_table_with_holes() {
        let table = vec![
            record! {
                "a" => Value::test_string("x"),
                "b" => Value::test_nothing(),
            },
            record! {
                "a" => Value::test_nothing(),
                "b" => Value::test_int(2),
            },
        ];

        let expected = (
            vec!["a".into(), "b".into()],
            vec!["string".into(), "int".into()],
            vec![vec!["x".into(), "".into()], vec!["".into(), "2".into()]],
        );

        assert_eq!(repr_table(&table), expected);
    }

    #[test]
    fn repr_table_with_mixed_numeric_types() {
        let table = vec![
            record! {
                "a" => Value::test_string("x"),
                "b" => Value::test_int(1),
            },
            record! {
                "a" => Value::test_string("y"),
                "b" => Value::test_float(2.34),
            },
        ];

        let expected = (
            vec!["a".into(), "b".into()],
            vec!["string".into(), "number".into()],
            vec![
                vec!["x".into(), "1".into()],
                vec!["y".into(), "2.34".into()],
            ],
        );

        assert_eq!(repr_table(&table), expected);
    }
}
