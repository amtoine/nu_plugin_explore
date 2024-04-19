use crossterm::event::KeyCode;
use nuon::{from_nuon, to_nuon};
use ratatui::{
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use nu_protocol::{Span, Value};

use crate::config::Config;

#[derive(Default, Clone)]
pub struct Editor {
    buffer: String,
    cursor_position: (usize, usize),
    width: usize,
}

#[derive(Debug, PartialEq)]
pub enum EditorTransition {
    Continue,
    Quit,
    Value(Value),
}

impl Editor {
    /// set the width of the editor
    ///
    /// this method removes the frame on the left and the right if any
    pub(super) fn set_width(&mut self, width: usize) {
        self.width = width - 2;
    }

    pub(super) fn from_value(value: &Value) -> Self {
        Self {
            // NOTE: `value` should be a valid [`Value`] and thus the conversion should never fail
            buffer: to_nuon(value, true, None, None, None).unwrap(),
            cursor_position: (0, 0),
            width: 0,
        }
    }

    fn position(&self) -> usize {
        let (x, y) = self.cursor_position;
        y * self.width + x
    }

    fn move_cursor_left(&mut self) {
        let position = self
            .position()
            .saturating_sub(1)
            .clamp(0, self.buffer.len());

        self.cursor_position = (position % self.width, position / self.width);
    }

    fn move_cursor_right(&mut self) {
        let position = self
            .position()
            .saturating_add(1)
            .clamp(0, self.buffer.len());

        self.cursor_position = (position % self.width, position / self.width);
    }

    fn move_cursor_up(&mut self) {
        let (x, y) = self.cursor_position;
        let y = y.saturating_sub(1).clamp(0, self.buffer.len() / self.width);

        self.cursor_position = (x, y);
    }

    fn move_cursor_down(&mut self) {
        let (x, y) = self.cursor_position;
        let y = y.saturating_add(1).clamp(0, self.buffer.len() / self.width);

        self.cursor_position = (x, y);

        if self.position() > self.buffer.len() {
            self.cursor_position = (
                self.buffer.len() % self.width,
                self.buffer.len() / self.width,
            );
        }
    }

    fn enter_char(&mut self, c: char) {
        self.buffer.insert(self.position(), c);
        self.move_cursor_right();
    }

    fn delete_char(&mut self, offset: i32) {
        let position = (self.position() as i32 + offset) as usize;

        // NOTE: work on the chars and do not use remove which works on bytes
        self.buffer = self
            .buffer
            .chars()
            .take(position)
            .chain(self.buffer.chars().skip(position + 1))
            .collect();
    }

    fn delete_char_before_cursor(&mut self) {
        let is_not_cursor_leftmost = self.position() != 0;

        if is_not_cursor_leftmost {
            self.delete_char(-1);
            self.move_cursor_left();
        }
    }

    fn delete_char_under_cursor(&mut self) {
        self.delete_char(0);
    }

    pub(super) fn handle_key(&mut self, key: &KeyCode) -> Result<EditorTransition, String> {
        match key {
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Up => self.move_cursor_up(),
            KeyCode::Down => self.move_cursor_down(),
            KeyCode::Char(c) => self.enter_char(*c),
            KeyCode::Backspace => self.delete_char_before_cursor(),
            KeyCode::Delete => self.delete_char_under_cursor(),
            KeyCode::Enter => match from_nuon(&self.buffer, Some(Span::unknown())) {
                Ok(val) => return Ok(EditorTransition::Value(val)),
                Err(err) => return Err(format!("could not convert back from NUON: {}", err)),
            },
            KeyCode::Esc => return Ok(EditorTransition::Quit),
            _ => {}
        }

        Ok(EditorTransition::Continue)
    }

    pub(super) fn render(&self, frame: &mut Frame, config: &Config) {
        let title = "Editor";

        let block = Paragraph::new(self.buffer.as_str())
            .style(
                Style::default()
                    .fg(config.colors.editor.buffer.foreground)
                    .bg(config.colors.editor.buffer.background),
            )
            .block(
                Block::default().borders(Borders::ALL).title(title).style(
                    Style::default()
                        .fg(config.colors.editor.frame.foreground)
                        .bg(config.colors.editor.frame.background),
                ),
            );

        let height = if self.buffer.is_empty() {
            1
        } else if (self.buffer.len() % self.width) == 0 {
            self.buffer.len() / self.width
        } else {
            self.buffer.len() / self.width + 1
        } as u16;
        let area = Rect {
            x: (frame.size().width - (self.width as u16 + 2)) / 2,
            y: frame.size().height - (height + 2) - 2,
            width: self.width as u16 + 2,
            height: height + 2,
        };

        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block.wrap(Wrap { trim: false }), area);

        let (x, y) = self.cursor_position;
        frame.set_cursor(area.x + 1 + (x as u16), area.y + 1 + (y as u16))
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use nu_protocol::Value;

    use super::{Editor, EditorTransition};

    #[test]
    fn edit_cells() {
        let mut editor = Editor::default();
        editor.set_width(10 + 2);
        editor.buffer = r#""""#.to_string();

        // NOTE: for the NUON conversion to work, the test string buffer needs to be wrapped in
        // parentheses.
        // in order not to make the strokes clunky, the quotes are added in the for loop below and
        // are implicite in the strokes below.
        let strokes = vec![
            (
                KeyCode::Enter,
                "",
                Ok(EditorTransition::Value(Value::test_string(""))),
            ),
            (KeyCode::Right, "", Ok(EditorTransition::Continue)),
            (KeyCode::Char('a'), "a", Ok(EditorTransition::Continue)),
            (KeyCode::Char('b'), "ab", Ok(EditorTransition::Continue)),
            (KeyCode::Char('c'), "abc", Ok(EditorTransition::Continue)),
            (KeyCode::Char('d'), "abcd", Ok(EditorTransition::Continue)),
            (KeyCode::Char('e'), "abcde", Ok(EditorTransition::Continue)),
            (KeyCode::Left, "abcde", Ok(EditorTransition::Continue)),
            (KeyCode::Char('f'), "abcdfe", Ok(EditorTransition::Continue)),
            (KeyCode::Left, "abcdfe", Ok(EditorTransition::Continue)),
            (KeyCode::Left, "abcdfe", Ok(EditorTransition::Continue)),
            (
                KeyCode::Char('g'),
                "abcgdfe",
                Ok(EditorTransition::Continue),
            ),
            (KeyCode::Right, "abcgdfe", Ok(EditorTransition::Continue)),
            (KeyCode::Right, "abcgdfe", Ok(EditorTransition::Continue)),
            (KeyCode::Right, "abcgdfe", Ok(EditorTransition::Continue)),
            (KeyCode::Up, "abcgdfe", Ok(EditorTransition::Continue)),
            (KeyCode::Down, "abcgdfe", Ok(EditorTransition::Continue)),
            (
                KeyCode::Char('h'),
                "abcgdfeh",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('i'),
                "abcgdfehi",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('j'),
                "abcgdfehij",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('k'),
                "abcgdfehijk",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('l'),
                "abcgdfehijkl",
                Ok(EditorTransition::Continue),
            ),
            (KeyCode::Up, "abcgdfehijkl", Ok(EditorTransition::Continue)),
            (
                KeyCode::Char('m'),
                "abmcgdfehijkl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Down,
                "abmcgdfehijkl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Left,
                "abmcgdfehijkl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('n'),
                "abmcgdfehijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Left,
                "abmcgdfehijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Left,
                "abmcgdfehijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Left,
                "abmcgdfehijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Left,
                "abmcgdfehijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Left,
                "abmcgdfehijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('o'),
                "abmcgdfeohijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Right,
                "abmcgdfeohijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Right,
                "abmcgdfeohijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Enter,
                "abmcgdfeohijknl",
                Ok(EditorTransition::Value(Value::test_string(
                    "abmcgdfeohijknl",
                ))),
            ),
            (
                KeyCode::Right,
                "abmcgdfeohijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Right,
                "abmcgdfeohijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Char('p'),
                "abmcgdfeohijkpnl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Backspace,
                "abmcgdfeohijknl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Backspace,
                "abmcgdfeohijnl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Backspace,
                "abmcgdfeohinl",
                Ok(EditorTransition::Continue),
            ),
            (KeyCode::Up, "abmcgdfeohinl", Ok(EditorTransition::Continue)),
            (
                KeyCode::Delete,
                "amcgdfeohinl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Delete,
                "acgdfeohinl",
                Ok(EditorTransition::Continue),
            ),
            (
                KeyCode::Delete,
                "agdfeohinl",
                Ok(EditorTransition::Continue),
            ),
            (KeyCode::Esc, "agdfeohinl", Ok(EditorTransition::Quit)),
            (
                KeyCode::Enter,
                "agdfeohinl",
                Ok(EditorTransition::Value(Value::test_string("agdfeohinl"))),
            ),
        ];

        for (key, expected_buffer, expected) in strokes {
            let result = editor.handle_key(&key);

            assert_eq!(result, expected);
            assert_eq!(editor.buffer, format!(r#""{}""#, expected_buffer));
        }
    }
}
