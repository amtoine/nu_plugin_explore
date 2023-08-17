use ratatui::{
    prelude::{Alignment, CrosstermBackend, Rect},
    style::Style,
    widgets::Paragraph,
    Frame,
};

use nu_protocol::ast::PathMember;
use nu_protocol::Value;

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
    if !state.bottom {
        data_path.pop();
    }

    let data_repr = match data.clone().follow_cell_path(&data_path, false) {
        Err(_) => panic!("unexpected error when following cell path during rendering"),
        Ok(Value::List { vals, .. }) => {
            if vals.is_empty() {
                "[list 0 item]".to_string()
            } else {
                vals.iter()
                    .map(render_value)
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        }
        Ok(Value::Record { cols, vals, .. }) => {
            if cols.is_empty() {
                "{record 0 field}".to_string()
            } else {
                cols.iter()
                    .zip(vals)
                    .map(|(col, val)| format!("{}: {}", col, render_value(&val)))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        }
        // FIXME: use a real config
        Ok(value) => value.into_string(" ", &nu_protocol::Config::default()),
    };

    frame.render_widget(Paragraph::new(data_repr), rect_without_bottom_bar);
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
        .fg(config.status_bar.foreground)
        .bg(config.status_bar.background);

    frame.render_widget(
        Paragraph::new(state.mode.to_string())
            .style(style)
            .alignment(Alignment::Left),
        bottom_bar_rect,
    );

    let hints = match state.mode {
        Mode::Normal => format!(
            "{} to {} | {}{}{}{} to move around",
            config.keybindings.insert,
            Mode::Insert,
            config.keybindings.navigation.left,
            config.keybindings.navigation.down,
            config.keybindings.navigation.up,
            config.keybindings.navigation.right,
        ),
        Mode::Insert => format!(
            "{} to {} | COMING SOON",
            config.keybindings.normal,
            Mode::Normal
        ),
    }
    .to_string();

    frame.render_widget(
        Paragraph::new(hints + &format!(" | {} to quit", config.keybindings.quit))
            .style(style)
            .alignment(Alignment::Right),
        bottom_bar_rect,
    );
}
