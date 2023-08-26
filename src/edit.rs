use console::Key;
use nu_protocol::{Span, Value};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{app::Mode, config::Config};

pub(super) struct Editor {
    pub buffer: String,
    cursor_position: usize,
}

#[allow(clippy::derivable_impls)]
impl Default for Editor {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            cursor_position: 0,
        }
    }
}

impl Editor {
    pub(super) fn from_value(value: &Value) -> Self {
        Self {
            buffer: value.into_string(" ", &nu_protocol::Config::default()),
            cursor_position: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        self.cursor_position = self
            .cursor_position
            .saturating_sub(1)
            .clamp(0, self.buffer.len());
    }

    fn move_cursor_right(&mut self) {
        self.cursor_position = self
            .cursor_position
            .saturating_add(1)
            .clamp(0, self.buffer.len());
    }

    fn enter_char(&mut self, c: char) {
        self.buffer.insert(self.cursor_position, c);
        self.move_cursor_right();
    }

    /// TODO: documentation
    fn delete_char(&mut self, offset: i32) {
        let position = self.cursor_position + (offset as usize);

        // NOTE: work on the chars and do not use remove which works on bytes
        self.buffer = self
            .buffer
            .chars()
            .take(position)
            .chain(self.buffer.chars().skip(position + 1))
            .collect();
    }

    fn delete_char_before_cursor(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;

        if is_not_cursor_leftmost {
            self.delete_char(-1);
            self.move_cursor_left();
        }
    }

    fn delete_char_under_cursor(&mut self) {
        self.delete_char(0);
    }

    /// TODO: documentation
    pub(super) fn handle_key(&mut self, key: &Key) -> Option<(Mode, Option<Value>)> {
        match key {
            Key::ArrowLeft => self.move_cursor_left(),
            Key::ArrowRight => self.move_cursor_right(),
            Key::Char(c) => self.enter_char(*c),
            Key::Backspace => self.delete_char_before_cursor(),
            Key::Del => self.delete_char_under_cursor(),
            Key::Enter => {
                let val = Value::String {
                    val: self.buffer.clone(),
                    span: Span::unknown(),
                };
                return Some((Mode::Normal, Some(val)));
            }
            Key::Escape => return Some((Mode::Normal, None)),
            _ => {}
        }

        None
    }

    pub(super) fn render(
        &self,
        frame: &mut Frame<CrosstermBackend<console::Term>>,
        config: &Config,
    ) {
        let block = Paragraph::new(self.buffer.as_str())
            .style(
                Style::default()
                    .fg(config.colors.editor.buffer.foreground)
                    .bg(config.colors.editor.buffer.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Editor")
                    .style(
                        Style::default()
                            .fg(config.colors.editor.frame.foreground)
                            .bg(config.colors.editor.frame.background),
                    ),
            );
        let area = centered_rect(50, 20, frame.size());

        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);

        frame.set_cursor(area.x + self.cursor_position as u16 + 1, area.y + 1)
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
