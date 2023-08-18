use ratatui::{
    prelude::{Alignment, CrosstermBackend, Rect},
    style::Style,
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

use nu_protocol::ast::PathMember;
use nu_protocol::Value;

use super::config::repr_keycode;
use super::{Config, Mode, State};

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

fn render_value(value: &Value) -> String {
    match value {
        Value::List { vals, .. } => {
            if vals.len() <= 1 {
                format!("[list {} item]", vals.len())
            } else {
                format!("[list {} items]", vals.len())
            }
        }
        Value::Record { cols, .. } => {
            if cols.len() <= 1 {
                format!("{{record {} field}}", cols.len())
            } else {
                format!("{{record {} fields}}", cols.len())
            }
        }
        // FIXME: use a real config
        value => value.into_string(" ", &nu_protocol::Config::default()),
    }
}

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

    let items: Vec<ListItem> = match data.clone().follow_cell_path(&data_path, false) {
        Err(_) => panic!("unexpected error when following cell path during rendering"),
        Ok(Value::List { vals, .. }) => {
            if vals.is_empty() {
                vec!["[list 0 item]".to_string()]
            } else {
                vals.iter().map(render_value).collect::<Vec<String>>()
            }
        }
        Ok(Value::Record { cols, vals, .. }) => {
            if cols.is_empty() {
                vec!["{record 0 field}".to_string()]
            } else {
                cols.iter()
                    .zip(vals)
                    .map(|(col, val)| format!("{}: {}", col, render_value(&val)))
                    .collect::<Vec<String>>()
            }
        }
        // FIXME: use a real config
        Ok(value) => vec![value.into_string(" ", &nu_protocol::Config::default())],
    }
    .iter()
    .map(|line| {
        ListItem::new(line.clone()).style(
            Style::default()
                .fg(config.colors.normal.foreground)
                .bg(config.colors.normal.background),
        )
    })
    .collect();

    let highlight_style = Style::default()
        .fg(config.colors.selected.foreground)
        .bg(config.colors.selected.background)
        .add_modifier(config.colors.selected_modifier);

    let items = List::new(items)
        .highlight_style(highlight_style)
        .highlight_symbol(&config.colors.selected_symbol);

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

    frame.render_stateful_widget(
        items,
        rect_without_bottom_bar,
        &mut ListState::default().with_selected(Some(selected)),
    )
}

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
                    PathMember::Int { val, .. } => format!("{}", val).to_string(),
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

fn render_status_bar(
    frame: &mut Frame<CrosstermBackend<console::Term>>,
    state: &State,
    config: &Config,
) {
    let bottom_bar_rect = Rect::new(0, frame.size().height - 1, frame.size().width, 1);
    let style = Style::default()
        .fg(config.colors.status_bar.foreground)
        .bg(config.colors.status_bar.background);

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
            "{} to peek all | {} to peek current view | {} to peek under cursor",
            repr_keycode(&config.keybindings.peeking.all),
            repr_keycode(&config.keybindings.peeking.current),
            repr_keycode(&config.keybindings.peeking.under),
        ),
    }
    .to_string();

    frame.render_widget(
        Paragraph::new(hints + &format!(" | {} to quit", repr_keycode(&config.keybindings.quit)))
            .style(style)
            .alignment(Alignment::Right),
        bottom_bar_rect,
    );
}
