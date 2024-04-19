//! the higher level application
use nu_protocol::{
    ast::{CellPath, PathMember},
    Span, Value,
};

use crate::{config::Config, edit::Editor};

/// the mode in which the application is
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Mode {
    /// the *navigation* mode, where the user can move around in the data
    #[default]
    Normal,
    /// lets the user edit cells of the structured data
    Insert,
    /// lets the user *peek* data out of the application, to be reused later
    Peeking,
    /// indicates that the user has arrived to the very bottom of the nested data, i.e. there is
    /// nothing more to the right
    Bottom,
    /// waits for more keys to perform an action, e.g. jumping to a line or motion repetition that
    /// both require to enter a number before the actual action
    Waiting(usize),
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Peeking => "PEEKING",
            Self::Bottom => "BOTTOM",
            Self::Waiting(_) => "WAITING",
        };
        write!(f, "{}", repr)
    }
}

#[derive(Clone)]
/// the complete state of the application
pub struct App {
    /// the full current path in the data
    pub position: CellPath,
    /// used for rendering
    pub rendering_tops: Vec<i32>,
    /// the current [`Mode`]
    pub mode: Mode,
    /// the editor to modify the cells of the data
    pub editor: Editor,
    /// the value that is being explored
    pub value: Value,
    /// the configuration for the app
    pub config: Config,
}

impl Default for App {
    fn default() -> Self {
        Self {
            position: CellPath { members: vec![] },
            rendering_tops: vec![],
            mode: Mode::default(),
            editor: Editor::default(),
            value: Value::default(),
            config: Config::default(),
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
            Value::Record { val: rec, .. } => {
                let cols = rec.columns().cloned().collect::<Vec<_>>();

                app.position.members.push(PathMember::String {
                    val: cols.first().unwrap_or(&"".to_string()).into(),
                    span: Span::unknown(),
                    optional: cols.is_empty(),
                })
            }
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

    pub(super) fn enter_editor(&mut self) {
        let value = self.value_under_cursor(None);

        self.mode = Mode::Insert;
        self.editor = Editor::from_value(&value);
    }

    pub(crate) fn value_under_cursor(&self, alternate_cursor: Option<CellPath>) -> Value {
        self.value
            .clone()
            .follow_cell_path(
                &alternate_cursor.unwrap_or(self.position.clone()).members,
                false,
            )
            .unwrap_or_else(|_| {
                panic!(
                    "unexpected error when following {:?} in {}",
                    self.position.members,
                    self.value
                        .to_expanded_string(" ", &nu_protocol::Config::default())
                )
            })
    }

    pub(crate) fn with_config(&self, config: Config) -> Self {
        let mut app = self.clone();
        app.config = config;
        app
    }
}
