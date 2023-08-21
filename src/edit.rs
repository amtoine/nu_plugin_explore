use nu_protocol::Value;
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub(super) struct Editor {
    buffer: String,
    cursor_position: usize,
}

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

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;

        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.buffer.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.buffer.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub(super) fn render(&self, frame: &mut Frame<CrosstermBackend<console::Term>>) {
        let block = Paragraph::new(self.buffer.as_str())
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Editor"));
        let area = centered_rect(60, 20, frame.size());

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
