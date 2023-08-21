//! the module responsible for rendering the TUI
use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Rect},
    style::{Modifier, Style},
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

    if state.mode == Mode::Insert {
        state.editor.render(frame);
    }
}

/// a common representation for an explore row
#[derive(Clone, Debug, PartialEq)]
struct DataRowRepr {
    name: Option<String>,
    shape: String,
    data: String,
}

impl DataRowRepr {
    #[allow(dead_code)]
    fn unnamed(data: impl Into<String>, shape: impl Into<String>) -> Self {
        Self {
            name: None,
            shape: shape.into(),
            data: data.into(),
        }
    }

    #[allow(dead_code)]
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

/// TODO: documentation
enum SpecialString {
    Url,
    Path,
}

impl std::fmt::Display for SpecialString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = match self {
            Self::Url => "url".to_string(),
            Self::Path => "path".to_string(),
        };
        write!(f, "{}", repr)
    }
}

/// TODO: documentation
impl SpecialString {
    fn parse(input: &str) -> Option<Self> {
        if let Ok(url) = url::Url::parse(input) {
            if url.scheme() == "file" {
                Some(Self::Path)
            } else {
                Some(Self::Url)
            }
        } else if input.contains('/') {
            Some(Self::Path)
        } else {
            None
        }
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

/// TODO: documentation
fn is_table(value: &Value, cell_path: &[PathMember]) -> Option<bool> {
    match value.clone().follow_cell_path(cell_path, false) {
        Ok(Value::List { vals, .. }) => {
            if vals.is_empty() {
                Some(false)
            } else {
                match vals[0] {
                    Value::Record { .. } => {
                        let first = vals[0].get_type().to_string();
                        Some(vals.iter().all(|v| v.get_type().to_string() == first))
                    }
                    _ => Some(false),
                }
            }
        }
        Ok(_) => Some(false),
        Err(_) => None,
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
    let current = if !state.is_at_bottom() {
        data_path.pop()
    } else {
        None
    };

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
            .unwrap_or(0),
        None => 0,
    };

    if is_table(data, &data_path).expect("cell path is invalid when checking for table") {
        let (header, rows) = match data
            .clone()
            .follow_cell_path(&data_path, false)
            .expect("cell path invalid when rendering table")
        {
            Value::List { vals, .. } => {
                let cols_with_type = vals[0]
                    .columns()
                    .iter()
                    .map(|c| {
                        let spans = vec![
                            Span::styled(
                                c.clone(),
                                Style::default()
                                    .fg(config.colors.normal.name.foreground)
                                    .bg(config.colors.normal.name.background),
                            ),
                            " (".into(),
                            Span::styled(
                                vals[0].get_data_by_key(c).unwrap().get_type().to_string(),
                                Style::default()
                                    .fg(config.colors.normal.shape.foreground)
                                    .bg(config.colors.normal.shape.background),
                            ),
                            ")".into(),
                        ];

                        Cell::from(Line::from(spans))
                    })
                    .collect::<Vec<Cell>>();

                let rows = vals
                    .iter()
                    .map(|v| {
                        v.columns()
                            .iter()
                            .map(|c| v.get_data_by_key(c).unwrap())
                            .collect()
                    })
                    .collect::<Vec<Vec<Value>>>();

                (cols_with_type, rows)
            }
            _ => panic!("value is a table but is not a list"),
        };

        let widths = header
            .iter()
            // FIXME: use an appropriate constraint here
            .map(|_| Constraint::Min(25))
            .collect::<Vec<Constraint>>();

        let header = Row::new(header).height(1);

        let rows: Vec<Row> = rows
            .iter()
            .map(|r| {
                let cells = r
                    .iter()
                    .map(|v| Cell::from(repr_value(v).data))
                    .collect::<Vec<Cell>>();

                Row::new(cells)
            })
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
            let items: Vec<ListItem> = repr_data(data, &data_path)
                .iter()
                .map(|row| {
                    let row = row.clone();

                    let mut spans = vec![];
                    if let Some(name) = row.name {
                        spans.push(Span::styled(
                            name,
                            Style::default()
                                .fg(config.colors.normal.name.foreground)
                                .bg(config.colors.normal.name.background),
                        ));
                        spans.push(": ".into());
                    }
                    spans.push("(".into());
                    spans.push(Span::styled(
                        row.shape,
                        Style::default()
                            .fg(config.colors.normal.shape.foreground)
                            .bg(config.colors.normal.shape.background),
                    ));
                    spans.push(") ".into());
                    spans.push(Span::styled(
                        row.data,
                        Style::default()
                            .fg(config.colors.normal.data.foreground)
                            .bg(config.colors.normal.data.background),
                    ));

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
            let header = Row::new(vec![
                Cell::from("name").style(
                    Style::default()
                        .fg(config.colors.normal.name.foreground)
                        .bg(config.colors.normal.name.background)
                        .add_modifier(Modifier::REVERSED),
                ),
                Cell::from("data").style(
                    Style::default()
                        .fg(config.colors.normal.data.foreground)
                        .bg(config.colors.normal.data.background)
                        .add_modifier(Modifier::REVERSED),
                ),
                Cell::from("shape").style(
                    Style::default()
                        .fg(config.colors.normal.shape.foreground)
                        .bg(config.colors.normal.shape.background)
                        .add_modifier(Modifier::REVERSED),
                ),
            ])
            .height(1);

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

            let table = if config.show_table_header {
                Table::new(rows).header(header)
            } else {
                Table::new(rows)
            }
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
        Mode::Bottom => Style::default()
            .fg(config.colors.status_bar.bottom.foreground)
            .bg(config.colors.status_bar.bottom.background),
    };

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
        Mode::Bottom => format!(
            "{} to {} | {} to peek",
            repr_keycode(&config.keybindings.navigation.left),
            Mode::Normal,
            repr_keycode(&config.keybindings.peek),
        ),
    };

    let left = Line::from(Span::styled(
        state.mode.to_string(),
        style.add_modifier(Modifier::REVERSED),
    ));
    let right = Line::from(Span::styled(
        hints + &format!(" | {} to quit", repr_keycode(&config.keybindings.quit)),
        style,
    ));

    frame.render_widget(
        Paragraph::new(left).alignment(Alignment::Left),
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
    use nu_protocol::Value;

    use super::{is_table, repr_data, repr_list, repr_record, repr_simple_value, DataRowRepr};

    #[test]
    fn simple_value() {
        #[rustfmt::skip]
        let cases = vec![
            (Value::test_string("foo"), DataRowRepr::unnamed("foo", "string")),
            (Value::test_int(1), DataRowRepr::unnamed("1", "int")),
            (Value::test_bool(true), DataRowRepr::unnamed("true", "bool")),
            (Value::test_nothing(), DataRowRepr::unnamed("", "nothing")),
            (Value::test_string("foo"), DataRowRepr::unnamed("foo", "string")),
            (Value::test_string("https://google.com"), DataRowRepr::unnamed("https://google.com", "url")),
            (Value::test_string("file:///some/file"), DataRowRepr::unnamed("file:///some/file", "path")),
            (Value::test_string("/path/to/something"), DataRowRepr::unnamed("/path/to/something", "path")),
            (Value::test_string("relative/path/"), DataRowRepr::unnamed("relative/path/", "path")),
            (Value::test_string("./relative/path/"), DataRowRepr::unnamed("./relative/path/", "path")),
            (Value::test_string("../../relative/path/"), DataRowRepr::unnamed("../../relative/path/", "path")),
            (Value::test_string("file:"), DataRowRepr::unnamed("file:", "path")),
            (
                Value::test_string("normal string with a / inside"),
                DataRowRepr::unnamed("normal string with a / inside", "path")
            ),
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
            (vec![], DataRowRepr::unnamed("[0 item]", "list")),
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
            (vec![], DataRowRepr::unnamed("{0 field}", "record")),
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

        let result = repr_data(&data, &[]);
        let expected: Vec<DataRowRepr> = vec![
            DataRowRepr::named("l", "[3 items]", "list"),
            DataRowRepr::named("r", "{2 fields}", "record"),
            DataRowRepr::named("s", "some string", "string"),
            DataRowRepr::named("i", "123", "int"),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn is_a_table() {
        #[rustfmt::skip]
        let table = Value::test_list(vec![
            Value::test_record(vec!["a", "b"], vec![Value::test_string("a"), Value::test_int(1)]),
            Value::test_record(vec!["a", "b"], vec![Value::test_string("a"), Value::test_int(1)]),
        ]);
        assert_eq!(is_table(&table, &[]), Some(true));

        #[rustfmt::skip]
        let not_a_table = Value::test_list(vec![
            Value::test_record(vec!["a"], vec![Value::test_string("a")]),
            Value::test_record(vec!["a", "b"], vec![Value::test_string("a"), Value::test_int(1)]),
        ]);
        assert_eq!(is_table(&not_a_table, &[]), Some(false));

        assert_eq!(is_table(&Value::test_int(0), &[]), Some(false));
    }
}
