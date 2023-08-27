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
    /// TODO: documentation
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
pub(super) struct App {
    /// the full current path in the data
    pub cell_path: CellPath,
    /// the current [`Mode`]
    pub mode: Mode,
    /// the editor to modify the cells of the data
    pub editor: Editor,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cell_path: CellPath { members: vec![] },
            mode: Mode::default(),
            editor: Editor::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub(super) fn from_value(value: &Value) -> Self {
        let mut app = Self::default();
        match value {
            Value::List { vals, .. } => app.cell_path.members.push(PathMember::Int {
                val: 0,
                span: Span::unknown(),
                optional: vals.is_empty(),
            }),
            Value::Record { cols, .. } => app.cell_path.members.push(PathMember::String {
                val: cols.get(0).unwrap_or(&"".to_string()).into(),
                span: Span::unknown(),
                optional: cols.is_empty(),
            }),
            _ => {}
        }

        app
    }

    /// TODO: documentation
    pub fn is_at_bottom(&self) -> bool {
        matches!(self.mode, Mode::Bottom)
    }

    /// TODO: documentation
    pub fn hit_bottom(&mut self) {
        self.mode = Mode::Bottom;
    }

    pub(super) fn enter_editor(&mut self, value: &Value) {
        self.mode = Mode::Insert;
        self.editor = Editor::from_value(value);
    }
}
