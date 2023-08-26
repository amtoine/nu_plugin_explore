use nu_protocol::{
    ast::{CellPath, PathMember},
    Span, Value,
};

use std::error;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

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

/// Application.
#[derive(Debug)]
pub struct App {
    /// the full current path in the data
    pub cell_path: CellPath,
    /// the current [`Mode`]
    pub mode: Mode,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cell_path: CellPath { members: vec![] },
            mode: Mode::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`State`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub fn from_value(value: &Value) -> Self {
        let mut state = Self::default();
        match value {
            Value::List { vals, .. } => state.cell_path.members.push(PathMember::Int {
                val: 0,
                span: Span::unknown(),
                optional: vals.is_empty(),
            }),
            Value::Record { cols, .. } => state.cell_path.members.push(PathMember::String {
                val: cols.get(0).unwrap_or(&"".to_string()).into(),
                span: Span::unknown(),
                optional: cols.is_empty(),
            }),
            _ => {}
        }

        state
    }

    /// TODO: documentation
    pub fn is_at_bottom(&self) -> bool {
        matches!(self.mode, Mode::Bottom)
    }

    /// TODO: documentation
    pub fn hit_bottom(&mut self) {
        self.mode = Mode::Bottom;
    }
}
