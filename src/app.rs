//! the higher level application
use nu_protocol::{
    ast::{CellPath, PathMember},
    Span, Value,
};

use crate::edit::Editor;

/// the mode in which the application is
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    /// the NORMAL mode is the *navigation* mode, where the user can move around in the data
    Normal,
    /// the INSERT mode lets the user edit cells of the structured data
    Insert,
    /// the PEEKING mode lets the user *peek* data out of the application, to be reused later
    Peeking,
    Bottom,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Peeking => "PEEKING",
            Self::Bottom => "BOTTOM",
        };
        write!(f, "{}", repr)
    }
}

/// the complete state of the application
pub struct App {
    /// the full current path in the data
    pub position: CellPath,
    /// the current [`Mode`]
    pub mode: Mode,
    /// the editor to modify the cells of the data
    pub editor: Editor,
    /// the value that is being explored
    pub value: Value,
}

impl Default for App {
    fn default() -> Self {
        Self {
            position: CellPath { members: vec![] },
            mode: Mode::default(),
            editor: Editor::default(),
            value: Value::default(),
        }
    }
}

impl App {
    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub(super) fn from_value(value: Value) -> Self {
        let mut app = Self::default();

        match &value {
            Value::List { vals, .. } => app.position.members.push(PathMember::Int {
                val: 0,
                span: Span::unknown(),
                optional: vals.is_empty(),
            }),
            Value::Record { val: rec, .. } => app.position.members.push(PathMember::String {
                val: rec.cols.first().unwrap_or(&"".to_string()).into(),
                span: Span::unknown(),
                optional: rec.cols.is_empty(),
            }),
            _ => {}
        }

        app.value = value;

        app
    }

    pub fn is_at_bottom(&self) -> bool {
        matches!(self.mode, Mode::Bottom)
    }

    pub fn hit_bottom(&mut self) {
        self.mode = Mode::Bottom;
    }

    pub(super) fn enter_editor(&mut self) -> Result<(), String> {
        let value = self
            .value
            .clone()
            .follow_cell_path(&self.position.members, false)
            .unwrap();

        if matches!(value, Value::String { .. }) {
            self.mode = Mode::Insert;
            self.editor = Editor::from_value(&value);

            Ok(())
        } else {
            // TODO: support more diverse cell edition
            Err(format!(
                "can only edit string cells, found {}",
                value.get_type()
            ))
        }
    }
}
