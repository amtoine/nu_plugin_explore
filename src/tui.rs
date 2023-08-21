//! the module responsible for rendering the TUI
use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame,
};

use nu_protocol::ast::PathMember;
use nu_protocol::Value;

use super::config::{repr_keycode, Layout};
use super::{Config, Mode, State};

/// render the whole ui
pub(super) fn render_ui(
    frame: &mut Frame<CrosstermBackend<console::Term>>,
    input: &Value,
    state: &State,
    config: &Config,
) {
    render_data(frame, input, state, config);
    if config.show_cell_path {
        render_cell_path(frame, state);
    }
    render_status_bar(frame, state, config);
}

/// a common representation for an explore row
#[derive(Clone, Debug, PartialEq)]
struct DataRowRepr {
    name: Option<String>,
    shape: String,
    data: String,
}

/// compute the preview representation of a list
///
/// > see the tests for detailed examples
fn repr_list(vals: &[Value]) -> DataRowRepr {
    let data = if vals.len() <= 1 {
        format!("[{} item]", vals.len())
    } else {
        format!("[{} items]", vals.len())
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
    let data = if cols.len() <= 1 {
        format!("{{{} field}}", cols.len())
    } else {
        format!("{{{} fields}}", cols.len())
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
    DataRowRepr {
        name: None,
        shape: value.get_type().to_string(),
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
        Value::Record { cols, .. } => repr_record(cols),
        x => repr_simple_value(x),
    }
}

/// compute the row / item representation of a complete Nushell Value
///
/// > see the tests for detailed examples
fn repr_data(data: &Value, cell_path: &[PathMember]) -> Vec<DataRowRepr> {
    match data.clone().follow_cell_path(cell_path, false) {
        Err(_) => panic!("unexpected error when following cell path during rendering"),
        Ok(Value::List { vals, .. }) => {
            if vals.is_empty() {
                vec![DataRowRepr {
                    name: None,
                    shape: "list".into(),
                    data: "[0 item]".into(),
                }]
            } else {
                vals.iter().map(repr_value).collect::<Vec<DataRowRepr>>()
            }
        }
        Ok(Value::Record { cols, vals, .. }) => {
            if cols.is_empty() {
                vec![DataRowRepr {
                    name: None,
                    shape: "record".into(),
                    data: "{0 field}".into(),
                }]
            } else {
                cols.iter()
                    .zip(vals)
                    .map(|(col, val)| {
                        let mut repr = repr_value(&val);
                        repr.name = Some(col.to_string());
                        repr
                    })
                    .collect::<Vec<DataRowRepr>>()
            }
        }
        Ok(value) => vec![repr_simple_value(&value)],
    }
}

/// render the whole data
///
/// the layout can be changed from [`crate::config::Config::layout`].
///
/// the data will be rendered on top of the bar, and on top of the cell path in case
/// [`crate::config::Config::show_cell_path`] is set to `true`.
fn render_data(
    frame: &mut Frame<CrosstermBackend<console::Term>>,
    data: &Value,
    state: &State,
    config: &Config,
) {
    let data_frame_height = if config.show_cell_path {
        frame.size().height - 2
    } else {
        frame.size().height - 1
    };
    let rect_without_bottom_bar = Rect::new(0, 0, frame.size().width, data_frame_height);

    let mut data_path = state.cell_path.members.clone();
    let current = if !state.bottom { data_path.pop() } else { None };

    let highlight_style = Style::default()
        .fg(config.colors.selected.foreground)
        .bg(config.colors.selected.background)
        .add_modifier(config.colors.selected_modifier);

    let selected = match current {
        Some(PathMember::Int { val, .. }) => val,
        Some(PathMember::String { val, .. }) => data
            .clone()
            .follow_cell_path(&data_path, false)
            .expect("unexpected error when following cell path during rendering")
            .columns()
            .iter()
            .position(|x| x == &val)
            .unwrap(),
        None => 0,
    };

    match config.layout {
        Layout::Compact => {
            let items: Vec<ListItem> = repr_data(data, &data_path)
                .iter()
                .map(|row| {
                    let row = row.clone();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            row.name.unwrap_or("".into()),
                            Style::default()
                                .fg(config.colors.normal.name.foreground)
                                .bg(config.colors.normal.name.background),
                        ),
                        ": (".into(),
                        Span::styled(
                            row.shape,
                            Style::default()
                                .fg(config.colors.normal.shape.foreground)
                                .bg(config.colors.normal.shape.background),
                        ),
                        ") ".into(),
                        Span::styled(
                            row.data,
                            Style::default()
                                .fg(config.colors.normal.data.foreground)
                                .bg(config.colors.normal.data.background),
                        ),
                    ]))
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
            let rows: Vec<Row> = repr_data(data, &data_path)
                .iter()
                .map(|row| {
                    let row = row.clone();
                    Row::new(vec![
                        Cell::from(row.name.unwrap_or("".into())).style(
                            Style::default()
                                .fg(config.colors.normal.name.foreground)
                                .bg(config.colors.normal.name.background),
                        ),
                        Cell::from(row.data).style(
                            Style::default()
                                .fg(config.colors.normal.data.foreground)
                                .bg(config.colors.normal.data.background),
                        ),
                        Cell::from(row.shape).style(
                            Style::default()
                                .fg(config.colors.normal.shape.foreground)
                                .bg(config.colors.normal.shape.background),
                        ),
                    ])
                })
                .collect();

            let table = Table::new(rows)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(highlight_style)
                .highlight_symbol(&config.colors.selected_symbol)
                .widths(&[
                    Constraint::Percentage(20),
                    Constraint::Percentage(70),
                    Constraint::Percentage(10),
                ]);

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
fn render_cell_path(frame: &mut Frame<CrosstermBackend<console::Term>>, state: &State) {
    let next_to_bottom_bar_rect = Rect::new(0, frame.size().height - 2, frame.size().width, 1);
    let cell_path = format!(
        "cell path: $.{}",
        state
            .cell_path
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
/// ||INSERT  ...                                                     <esc> to NORMAL | COMING SOON | q to quit||
/// ```
/// - in PEEKING mode
/// ```text
/// ||PEEKING ... <esc> to NORMAL | a to peek all | c to peek current view | u to peek under cursor | q to quit||
/// ```
fn render_status_bar(
    frame: &mut Frame<CrosstermBackend<console::Term>>,
    state: &State,
    config: &Config,
) {
    let bottom_bar_rect = Rect::new(0, frame.size().height - 1, frame.size().width, 1);

    let style = match state.mode {
        Mode::Normal => Style::default()
            .fg(config.colors.status_bar.normal.foreground)
            .bg(config.colors.status_bar.normal.background),
        Mode::Insert => Style::default()
            .fg(config.colors.status_bar.insert.foreground)
            .bg(config.colors.status_bar.insert.background),
        Mode::Peeking => Style::default()
            .fg(config.colors.status_bar.peek.foreground)
            .bg(config.colors.status_bar.peek.background),
    };

    frame.render_widget(
        Paragraph::new(state.mode.to_string())
            .style(style)
            .alignment(Alignment::Left),
        bottom_bar_rect,
    );

    let hints = match state.mode {
        Mode::Normal => format!(
            "{} to {} | {}{}{}{} to move around | {} to peek",
            repr_keycode(&config.keybindings.insert),
            Mode::Insert,
            repr_keycode(&config.keybindings.navigation.left),
            repr_keycode(&config.keybindings.navigation.down),
            repr_keycode(&config.keybindings.navigation.up),
            repr_keycode(&config.keybindings.navigation.right),
            repr_keycode(&config.keybindings.peek),
        ),
        Mode::Insert => format!(
            "{} to {} | COMING SOON",
            repr_keycode(&config.keybindings.normal),
            Mode::Normal
        ),
        Mode::Peeking => format!(
            "{} to {} | {} to peek all | {} to peek current view | {} to peek under cursor",
            repr_keycode(&config.keybindings.normal),
            Mode::Normal,
            repr_keycode(&config.keybindings.peeking.all),
            repr_keycode(&config.keybindings.peeking.current),
            repr_keycode(&config.keybindings.peeking.under),
        ),
    };

    frame.render_widget(
        Paragraph::new(hints + &format!(" | {} to quit", repr_keycode(&config.keybindings.quit)))
            .style(style)
            .alignment(Alignment::Right),
        bottom_bar_rect,
    );
}

// TODO: add proper assert error messages
#[cfg(test)]
mod tests {
    use nu_protocol::Value;

    use crate::config::{Config, Layout};

    use super::{repr_data, repr_list, repr_record, repr_simple_value};

    #[test]
    fn simple_value() {
        let mut config = Config::default();

        #[rustfmt::skip]
        let cases = vec![
            (Layout::Table, Value::test_string("foo"), vec!["foo", "string"]),
            (Layout::Compact, Value::test_string("foo"), vec!["(string) foo"]),
            (Layout::Table, Value::test_int(1), vec!["1", "int"]),
            (Layout::Compact, Value::test_int(1), vec!["(int) 1"]),
            (Layout::Table, Value::test_bool(true), vec!["true", "bool"]),
            (Layout::Compact, Value::test_bool(true), vec!["(bool) true"]),
            (Layout::Table, Value::test_nothing(), vec!["", "nothing"]),
            (Layout::Compact, Value::test_nothing(), vec!["(nothing) "]),
        ];

        for (layout, value, expected) in cases {
            config.layout = layout;
            let result = repr_simple_value(&value, &config);
            let expected: Vec<String> = expected.iter().map(|x| x.to_string()).collect();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn list() {
        let mut config = Config::default();

        let list = vec![
            Value::test_string("a"),
            Value::test_int(1),
            Value::test_bool(false),
        ];

        #[rustfmt::skip]
        let cases = vec![
            (Layout::Table, list.clone(), vec!["[3 items]", "list"]),
            (Layout::Compact, list.clone(), vec!["[list 3 items]"]),
            (Layout::Table, vec![], vec!["[0 item]", "list"]),
            (Layout::Compact, vec![], vec!["[list 0 item]"]),
            (Layout::Table, vec![Value::test_nothing()], vec!["[1 item]", "list"]),
            (Layout::Compact, vec![Value::test_nothing()], vec!["[list 1 item]"]),
        ];

        for (layout, list, expected) in cases {
            config.layout = layout;
            let result = repr_list(&list, &config);
            let expected: Vec<String> = expected.iter().map(|x| x.to_string()).collect();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn record() {
        let mut config = Config::default();

        #[rustfmt::skip]
        let cases = vec![
            (Layout::Table, vec!["a", "b", "c"], vec!["{3 fields}", "record"]),
            (Layout::Compact, vec!["a", "b", "c"], vec!["{record 3 fields}"]),
            (Layout::Table, vec![], vec!["{0 field}", "record"]),
            (Layout::Compact, vec![], vec!["{record 0 field}"]),
            (Layout::Table, vec!["a"], vec!["{1 field}", "record"]),
            (Layout::Compact, vec!["a"], vec!["{record 1 field}"]),
        ];

        for (layout, record, expected) in cases {
            config.layout = layout;
            let result = repr_record(
                &record.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
                &config,
            );
            let expected: Vec<String> = expected.iter().map(|x| x.to_string()).collect();
            assert_eq!(result, expected);
        }
    }

    #[ignore = "repr_value is just a direct wrapper around repr_list, repr_record and repr_simple_value"]
    #[test]
    fn value() {}

    #[test]
    fn data() {
        let mut config = Config::default();

        let data = Value::test_record(
            vec!["l", "r", "s", "i"],
            vec![
                Value::test_list(vec![
                    Value::test_string("my"),
                    Value::test_string("list"),
                    Value::test_string("elements"),
                ]),
                Value::test_record(vec!["a", "b"], vec![Value::test_int(1), Value::test_int(2)]),
                Value::test_string("some string"),
                Value::test_int(123),
            ],
        );

        config.layout = Layout::Table;
        let result = repr_data(&data, &[], &config);
        let expected: Vec<Vec<String>> = vec![
            vec!["l".into(), "[3 items]".into(), "list".into()],
            vec!["r".into(), "{2 fields}".into(), "record".into()],
            vec!["s".into(), "some string".into(), "string".into()],
            vec!["i".into(), "123".into(), "int".into()],
        ];
        assert_eq!(result, expected);

        config.layout = Layout::Compact;
        let result = repr_data(&data, &[], &config);
        let expected: Vec<Vec<String>> = vec![vec![
            "l: [list 3 items]".into(),
            "r: {record 2 fields}".into(),
            "s: (string) some string".into(),
            "i: (int) 123".into(),
        ]];
        assert_eq!(result, expected);
    }
}
