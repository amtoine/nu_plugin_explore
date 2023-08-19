//! the module responsible for rendering the TUI
use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Rect},
    style::Style,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState},
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

/// compute the preview representation of a list
///
/// > see the tests for detailed examples
fn repr_list(vals: &[Value], config: &Config) -> Vec<String> {
    if vals.len() <= 1 {
        match config.layout {
            Layout::Compact => vec![format!("[list {} item]", vals.len())],
            Layout::Table => vec![format!("[{} item]", vals.len()), "list".to_string()],
        }
    } else {
        match config.layout {
            Layout::Compact => vec![format!("[list {} items]", vals.len())],
            Layout::Table => vec![format!("[{} items]", vals.len()), "list".to_string()],
        }
    }
}

/// compute the preview representation of a record
///
/// > see the tests for detailed examples
fn repr_record(cols: &[String], config: &Config) -> Vec<String> {
    if cols.len() <= 1 {
        match config.layout {
            Layout::Compact => vec![format!("{{record {} field}}", cols.len())],
            Layout::Table => {
                vec![format!("{{{} field}}", cols.len()), "record".to_string()]
            }
        }
    } else {
        match config.layout {
            Layout::Compact => vec![format!("{{record {} fields}}", cols.len())],
            Layout::Table => {
                vec![format!("{{{} fields}}", cols.len()), "record".to_string()]
            }
        }
    }
}

/// compute the preview representation of a simple value
///
/// > see the tests for detailed examples
fn repr_simple_value(value: &Value, config: &Config) -> Vec<String> {
    // FIXME: use a real config
    match config.layout {
        Layout::Compact => vec![format!(
            "({}) {}",
            value.get_type(),
            value.into_string(" ", &nu_protocol::Config::default())
        )],
        Layout::Table => vec![
            value.into_string(" ", &nu_protocol::Config::default()),
            value.get_type().to_string(),
        ],
    }
}

/// compute the preview representation of a value
///
/// > see the tests for detailed examples
fn repr_value(value: &Value, config: &Config) -> Vec<String> {
    match value {
        Value::List { vals, .. } => repr_list(vals, config),
        Value::Record { cols, .. } => repr_record(cols, config),
        x => repr_simple_value(x, config),
    }
}

/// compute the row / item representation of a complete Nushell Value
///
/// > see the tests for detailed examples
fn repr_data(data: &Value, cell_path: &[PathMember], config: &Config) -> Vec<Vec<String>> {
    match data.clone().follow_cell_path(cell_path, false) {
        Err(_) => panic!("unexpected error when following cell path during rendering"),
        Ok(Value::List { vals, .. }) => match config.layout {
            Layout::Compact => {
                vec![if vals.is_empty() {
                    vec!["[list 0 item]".to_string()]
                } else {
                    vals.iter()
                        .map(|v| repr_value(v, config)[0].clone())
                        .collect::<Vec<String>>()
                }]
            }
            Layout::Table => {
                if vals.is_empty() {
                    vec![vec!["".into(), "[0 item]".into(), "list".into()]]
                } else {
                    vals.iter()
                        .map(|val| {
                            let mut value = repr_value(val, config);
                            if value.len() < 3 {
                                let mut base = vec![];
                                for _ in 0..(3 - value.len()) {
                                    base.push("".into())
                                }
                                base.append(&mut value);
                                base
                            } else {
                                value
                            }
                        })
                        .collect::<Vec<Vec<String>>>()
                }
            }
        },
        Ok(Value::Record { cols, vals, .. }) => match config.layout {
            Layout::Compact => {
                vec![if cols.is_empty() {
                    vec!["{record 0 field}".to_string()]
                } else {
                    cols.iter()
                        .zip(vals)
                        .map(|(col, val)| format!("{}: {}", col, repr_value(&val, config)[0]))
                        .collect::<Vec<String>>()
                }]
            }
            Layout::Table => {
                if cols.is_empty() {
                    vec![vec!["".into(), "{0 field}".into(), "record".into()]]
                } else {
                    cols.iter()
                        .zip(vals)
                        .map(|(col, val)| {
                            let mut res = vec![col.clone()];
                            res.append(&mut repr_value(&val, config));
                            res
                        })
                        .collect::<Vec<Vec<String>>>()
                }
            }
        },
        // FIXME: use a real config
        Ok(value) => match config.layout {
            Layout::Compact => vec![vec![format!(
                "({}) {}",
                value.get_type(),
                value.into_string(" ", &nu_protocol::Config::default())
            )]],
            Layout::Table => vec![vec![
                "".into(),
                value.into_string(" ", &nu_protocol::Config::default()),
                value.get_type().to_string(),
            ]],
        },
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

    let normal_style = Style::default()
        .fg(config.colors.normal.foreground)
        .bg(config.colors.normal.background);
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
            let items: Vec<ListItem> = repr_data(data, &data_path, config)[0]
                .clone()
                .iter()
                .map(|line| ListItem::new(line.clone()).style(normal_style))
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
            let rows: Vec<Row> = repr_data(data, &data_path, config)
                .iter()
                .map(|row| Row::new(row.clone()).style(normal_style))
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
