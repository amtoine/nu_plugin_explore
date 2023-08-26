use console::Key;
use nu_protocol::{Span, Value};
use ratatui::{
    prelude::{CrosstermBackend, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{app::Mode, config::Config};

pub(super) struct Editor {
    pub buffer: String,
    cursor_position: (usize, usize),
    width: usize,
}

#[allow(clippy::derivable_impls)]
impl Default for Editor {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            cursor_position: (0, 0),
            width: 0,
        }
    }
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
            buffer: value.into_string(" ", &nu_protocol::Config::default()),
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

    /// TODO: documentation
    fn delete_char(&mut self, offset: i32) {
        let position = self.position() + (offset as usize);

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

    /// TODO: documentation
    pub(super) fn handle_key(&mut self, key: &Key) -> Option<(Mode, Option<Value>)> {
        match key {
            Key::ArrowLeft => self.move_cursor_left(),
            Key::ArrowRight => self.move_cursor_right(),
            Key::ArrowUp => self.move_cursor_up(),
            Key::ArrowDown => self.move_cursor_down(),
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
    use console::Key;
    use nu_protocol::Value;

    use crate::app::Mode;

    use super::Editor;

    #[test]
    fn edit_cells() {
        let mut editor = Editor::default();
        editor.set_width(10 + 2);

        let strokes = vec![
            (
                Key::Enter,
                "",
                Some((Mode::Normal, Some(Value::test_string("")))),
            ),
            (Key::Char('a'), "a", None),
            (Key::Char('b'), "ab", None),
            (Key::Char('c'), "abc", None),
            (Key::Char('d'), "abcd", None),
            (Key::Char('e'), "abcde", None),
            (Key::ArrowLeft, "abcde", None),
            (Key::Char('f'), "abcdfe", None),
            (Key::ArrowLeft, "abcdfe", None),
            (Key::ArrowLeft, "abcdfe", None),
            (Key::Char('g'), "abcgdfe", None),
            (Key::ArrowRight, "abcgdfe", None),
            (Key::ArrowRight, "abcgdfe", None),
            (Key::ArrowRight, "abcgdfe", None),
            (Key::ArrowUp, "abcgdfe", None),
            (Key::ArrowDown, "abcgdfe", None),
            (Key::Char('h'), "abcgdfeh", None),
            (Key::Char('i'), "abcgdfehi", None),
            (Key::Char('j'), "abcgdfehij", None),
            (Key::Char('k'), "abcgdfehijk", None),
            (Key::Char('l'), "abcgdfehijkl", None),
            (Key::ArrowUp, "abcgdfehijkl", None),
            (Key::Char('m'), "abmcgdfehijkl", None),
            (Key::ArrowDown, "abmcgdfehijkl", None),
            (Key::ArrowLeft, "abmcgdfehijkl", None),
            (Key::Char('n'), "abmcgdfehijknl", None),
            (Key::ArrowLeft, "abmcgdfehijknl", None),
            (Key::ArrowLeft, "abmcgdfehijknl", None),
            (Key::ArrowLeft, "abmcgdfehijknl", None),
            (Key::ArrowLeft, "abmcgdfehijknl", None),
            (Key::ArrowLeft, "abmcgdfehijknl", None),
            (Key::Char('o'), "abmcgdfeohijknl", None),
            (Key::ArrowRight, "abmcgdfeohijknl", None),
            (Key::ArrowRight, "abmcgdfeohijknl", None),
            (
                Key::Enter,
                "abmcgdfeohijknl",
                Some((Mode::Normal, Some(Value::test_string("abmcgdfeohijknl")))),
            ),
            (Key::ArrowRight, "abmcgdfeohijknl", None),
            (Key::ArrowRight, "abmcgdfeohijknl", None),
            (Key::Char('p'), "abmcgdfeohijkpnl", None),
            (Key::Backspace, "abmcgdfeohijknl", None),
            (Key::Backspace, "abmcgdfeohijnl", None),
            (Key::Backspace, "abmcgdfeohinl", None),
            (Key::ArrowUp, "abmcgdfeohinl", None),
            (Key::Del, "amcgdfeohinl", None),
            (Key::Del, "acgdfeohinl", None),
            (Key::Del, "agdfeohinl", None),
            (Key::Escape, "agdfeohinl", Some((Mode::Normal, None))),
            (
                Key::Enter,
                "agdfeohinl",
                Some((Mode::Normal, Some(Value::test_string("agdfeohinl")))),
            ),
        ];

        for (key, expected_buffer, expected) in strokes {
            let result = editor.handle_key(&key);

            assert_eq!(result, expected);
            assert_eq!(editor.buffer, expected_buffer.to_string());
        }
    }
}
