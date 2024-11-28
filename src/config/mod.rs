//! management of the outside configuration of `explore`
//!
//! this module
//! 1. holds the data structure of the [`Config`]
//! 1. gives default values to a [`Config`] with [`Config::default`]
//! 1. parses a Nushell [`Value`](https://docs.rs/nu-protocol/0.83.1/nu_protocol/enum.Value.html) into a valid [`Config`]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier};

use nu_protocol::{LabeledError, Span, Value};

mod parsing;
use parsing::{
    follow_cell_path, invalid_field, invalid_type, positive_integer, try_bool, try_fg_bg_colors,
    try_int, try_key, try_layout, try_modifier, try_string,
};

/// the configuration for the status bar colors in all [`crate::app::Mode`]s
#[derive(Clone, PartialEq, Debug)]
pub struct StatusBarColorConfig {
    pub normal: BgFgColorConfig,
    pub insert: BgFgColorConfig,
    pub peek: BgFgColorConfig,
    pub bottom: BgFgColorConfig,
}

/// the configuration for a row of the data rendering table
#[derive(Clone, PartialEq, Debug)]
pub struct TableRowColorConfig {
    // the name of the data, i.e. the key on the left
    pub name: BgFgColorConfig,
    // the data itself,
    pub data: BgFgColorConfig,
    // the type of the data, e.g. `string` or `int`
    pub shape: BgFgColorConfig,
}

/// the configuration for the editor box
#[derive(Clone, PartialEq, Debug)]
pub struct EditorColorConfig {
    pub frame: BgFgColorConfig,
    pub buffer: BgFgColorConfig,
}

/// the configuration for the line numbers
#[derive(Clone, PartialEq, Debug)]
pub struct LineNumbersColorConfig {
    // all lines
    pub normal: BgFgColorConfig,
    // the selected line
    pub selected: BgFgColorConfig,
}

/// the colors of the application
#[derive(Clone, PartialEq, Debug)]
pub struct ColorConfig {
    /// the color when a row is NOT selected
    pub normal: TableRowColorConfig,
    /// the color when a row is selected
    pub selected: BgFgColorConfig,
    /// the modifier to apply to the row under the cursor
    pub selected_modifier: Modifier,
    /// the symbol to show to the left of the selected row under the cursor
    pub selected_symbol: String,
    pub status_bar: StatusBarColorConfig,
    /// the color when editing a cell
    pub editor: EditorColorConfig,
    /// the color of a warning banner
    pub warning: BgFgColorConfig,
    /// the color of the line numbers
    pub line_numbers: LineNumbersColorConfig,
}

/// a pair of background / foreground colors
#[derive(Clone, Debug, PartialEq)]
pub struct BgFgColorConfig {
    pub background: Color,
    pub foreground: Color,
}

/// the bindings in NORMAL mode (see [crate::app::Mode::Normal])
#[derive(Clone, PartialEq, Debug)]
pub struct NavigationBindingsMap {
    /// go one row up in the data
    pub up: KeyEvent,
    /// go one row down in the data
    pub down: KeyEvent,
    /// go one level higher in the data
    pub left: KeyEvent,
    /// go one level deeper in the data
    pub right: KeyEvent,
    /// go one half page up in the data
    pub half_page_up: KeyEvent,
    /// go one half page down in the data
    pub half_page_down: KeyEvent,
    /// go to the top of the data, i.e. the first element or the first key
    pub goto_top: KeyEvent,
    /// go to the bottom of the data, i.e. the last element or the last key
    pub goto_bottom: KeyEvent,
    /// go at a particular line in the data
    pub goto_line: KeyEvent,
}

/// the bindings in PEEKING mode (see [crate::app::Mode::Peeking])
#[derive(Clone, PartialEq, Debug)]
pub struct PeekingBindingsMap {
    /// peek the whole data structure
    pub all: KeyEvent,
    /// peek the current cell path
    pub cell_path: KeyEvent,
    /// peek the current level, but only the row under the cursor
    pub under: KeyEvent,
    /// peek the current view
    pub view: KeyEvent,
}

/// the keybindings mapping
#[derive(Clone, PartialEq, Debug)]
pub struct KeyBindingsMap {
    pub quit: KeyEvent,
    /// go into INSERT mode (see [crate::app::Mode::Insert])
    pub insert: KeyEvent,
    /// go back into NORMAL mode (see [crate::app::Mode::Normal])
    pub normal: KeyEvent,
    pub navigation: NavigationBindingsMap,
    /// go into PEEKING mode (see [crate::app::Mode::Peeking])
    pub peek: KeyEvent,
    pub peeking: PeekingBindingsMap,
    pub transpose: KeyEvent,
}

/// the layout of the application
#[derive(Clone, PartialEq, Debug)]
pub enum Layout {
    /// show each row in a `[name, data, type]` column
    Table,
    /// show each row in compact form, to the left, `"{name}: ({type}) {data}"`
    Compact,
}

/// the configuration of the whole application
#[derive(Clone, PartialEq, Debug)]
pub struct Config {
    pub colors: ColorConfig,
    pub keybindings: KeyBindingsMap,
    pub show_cell_path: bool,
    pub layout: Layout,
    pub show_table_header: bool,
    pub margin: usize,
    pub number: bool,
    pub relativenumber: bool,
    pub show_hints: bool,
    pub strict_tables: bool,
}

impl Default for Config {
    fn default() -> Self {
        // "reset" is used instead of "black" in a dark terminal because, when the terminal is actually
        // black, "black" is not really black which is ugly, whereas "reset" is really black.
        Self {
            show_cell_path: true,
            show_table_header: true,
            layout: Layout::Table,
            margin: 10,
            number: false,
            relativenumber: false,
            show_hints: true,
            strict_tables: false,
            colors: ColorConfig {
                normal: TableRowColorConfig {
                    name: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::Green,
                    },
                    data: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::White,
                    },
                    shape: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::Blue,
                    },
                },
                selected: BgFgColorConfig {
                    background: Color::White,
                    foreground: Color::Black,
                },
                selected_modifier: Modifier::BOLD,
                selected_symbol: "".into(),
                status_bar: StatusBarColorConfig {
                    normal: BgFgColorConfig {
                        background: Color::Black,
                        foreground: Color::White,
                    },
                    insert: BgFgColorConfig {
                        background: Color::Black,
                        foreground: Color::LightYellow,
                    },
                    peek: BgFgColorConfig {
                        background: Color::Black,
                        foreground: Color::LightGreen,
                    },
                    bottom: BgFgColorConfig {
                        background: Color::Black,
                        foreground: Color::LightMagenta,
                    },
                },
                editor: EditorColorConfig {
                    frame: BgFgColorConfig {
                        background: Color::Black,
                        foreground: Color::LightCyan,
                    },
                    buffer: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::White,
                    },
                },
                warning: BgFgColorConfig {
                    background: Color::Yellow,
                    foreground: Color::Red,
                },
                line_numbers: LineNumbersColorConfig {
                    normal: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::White,
                    },
                    selected: BgFgColorConfig {
                        background: Color::White,
                        foreground: Color::Black,
                    },
                },
            },
            keybindings: KeyBindingsMap {
                quit: KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                insert: KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
                normal: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                navigation: NavigationBindingsMap {
                    left: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                    down: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                    up: KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
                    right: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                    half_page_down: KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                    half_page_up: KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                    goto_top: KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
                    goto_bottom: KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE),
                    goto_line: KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
                },
                peek: KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
                peeking: PeekingBindingsMap {
                    all: KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                    cell_path: KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
                    under: KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
                    view: KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE),
                },
                transpose: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            },
        }
    }
}

impl Config {
    // NOTE: all the _unwraps_ called on the output of [`parsing::follow_cell_path`] are safe
    // because they all follow the same cell path as the parsing branch they are in, e.g.
    // `follow_cell_path(&value, &["colors", "line_numbers"])` is only found in the "colors" and
    // "line_numbers" branch of the parsing.
    pub fn from_value(value: &Value) -> Result<Self, LabeledError> {
        let mut config = Config::default();

        for column in value.columns() {
            match column.as_str() {
                "show_cell_path" => {
                    if let Some(val) = try_bool(value, &["show_cell_path"])? {
                        config.show_cell_path = val
                    }
                }
                "show_table_header" => {
                    if let Some(val) = try_bool(value, &["show_table_header"])? {
                        config.show_table_header = val
                    }
                }
                "layout" => {
                    if let Some(val) = try_layout(value, &["layout"])? {
                        config.layout = val
                    }
                }
                "margin" => {
                    if let Some(val) = try_int(value, &["margin"])? {
                        if val < 0 {
                            return Err(positive_integer(val, &["margin"], Span::unknown()));
                        }
                        config.margin = val as usize
                    }
                }
                "number" => {
                    if let Some(val) = try_bool(value, &["number"])? {
                        config.number = val
                    }
                }
                "relativenumber" => {
                    if let Some(val) = try_bool(value, &["relativenumber"])? {
                        config.relativenumber = val
                    }
                }
                "show_hints" => {
                    if let Some(val) = try_bool(value, &["show_hints"])? {
                        config.show_hints = val
                    }
                }
                "strict_tables" => {
                    if let Some(val) = try_bool(value, &["strict_tables"])? {
                        config.strict_tables = val
                    }
                }
                "colors" => {
                    let cell = follow_cell_path(value, &["colors"]).unwrap();
                    let columns = match &cell {
                        Value::Record { val: rec, .. } => rec.columns().collect::<Vec<_>>(),
                        x => return Err(invalid_type(x, &["colors"], "record")),
                    };

                    for column in columns {
                        match column.as_str() {
                            "normal" => {
                                let cell = follow_cell_path(value, &["colors", "normal"]).unwrap();
                                let columns = match &cell {
                                    Value::Record { val: rec, .. } => {
                                        rec.columns().collect::<Vec<_>>()
                                    }
                                    x => {
                                        return Err(invalid_type(
                                            x,
                                            &["colors", "normal"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "name" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "normal", "name"],
                                                &config.colors.normal.name,
                                            )? {
                                                config.colors.normal.name = val
                                            }
                                        }
                                        "data" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "normal", "data"],
                                                &config.colors.normal.data,
                                            )? {
                                                config.colors.normal.data = val
                                            }
                                        }
                                        "shape" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "normal", "shape"],
                                                &config.colors.normal.shape,
                                            )? {
                                                config.colors.normal.shape = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["colors", "normal", x],
                                                cell.span(),
                                            ))
                                        }
                                    }
                                }
                            }
                            "selected" => {
                                if let Some(val) = try_fg_bg_colors(
                                    value,
                                    &["colors", "selected"],
                                    &config.colors.selected,
                                )? {
                                    config.colors.selected = val
                                }
                            }
                            "selected_symbol" => {
                                if let Some(val) =
                                    try_string(value, &["colors", "selected_symbol"])?
                                {
                                    config.colors.selected_symbol = val
                                }
                            }
                            "selected_modifier" => {
                                if let Some(val) =
                                    try_modifier(value, &["colors", "selected_modifier"])?
                                {
                                    config.colors.selected_modifier = val
                                }
                            }
                            "status_bar" => {
                                let cell =
                                    follow_cell_path(value, &["colors", "status_bar"]).unwrap();
                                let columns = match &cell {
                                    Value::Record { val: rec, .. } => {
                                        rec.columns().collect::<Vec<_>>()
                                    }
                                    x => {
                                        return Err(invalid_type(
                                            x,
                                            &["colors", "status_bar"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "normal" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "status_bar", "normal"],
                                                &config.colors.status_bar.normal,
                                            )? {
                                                config.colors.status_bar.normal = val
                                            }
                                        }
                                        "insert" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "status_bar", "insert"],
                                                &config.colors.status_bar.insert,
                                            )? {
                                                config.colors.status_bar.insert = val
                                            }
                                        }
                                        "peek" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "status_bar", "peek"],
                                                &config.colors.status_bar.peek,
                                            )? {
                                                config.colors.status_bar.peek = val
                                            }
                                        }
                                        "bottom" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "status_bar", "bottom"],
                                                &config.colors.status_bar.bottom,
                                            )? {
                                                config.colors.status_bar.bottom = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["colors", "status_bar", x],
                                                cell.span(),
                                            ))
                                        }
                                    }
                                }
                            }
                            "editor" => {
                                let cell = follow_cell_path(value, &["colors", "editor"]).unwrap();
                                let columns = match &cell {
                                    Value::Record { val: rec, .. } => {
                                        rec.columns().collect::<Vec<_>>()
                                    }
                                    x => {
                                        return Err(invalid_type(
                                            x,
                                            &["colors", "editor"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "frame" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "editor", "frame"],
                                                &config.colors.editor.frame,
                                            )? {
                                                config.colors.editor.frame = val
                                            }
                                        }
                                        "buffer" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "editor", "buffer"],
                                                &config.colors.editor.buffer,
                                            )? {
                                                config.colors.editor.buffer = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["colors", "editor", x],
                                                cell.span(),
                                            ))
                                        }
                                    }
                                }
                            }
                            "warning" => {
                                if let Some(val) = try_fg_bg_colors(
                                    value,
                                    &["colors", "warning"],
                                    &config.colors.warning,
                                )? {
                                    config.colors.warning = val
                                }
                            }
                            "line_numbers" => {
                                let cell =
                                    follow_cell_path(value, &["colors", "line_numbers"]).unwrap();
                                let columns = match &cell {
                                    Value::Record { val: rec, .. } => {
                                        rec.columns().collect::<Vec<_>>()
                                    }
                                    x => {
                                        return Err(invalid_type(
                                            x,
                                            &["colors", "line_numbers"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "normal" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "line_numbers", "normal"],
                                                &config.colors.line_numbers.normal,
                                            )? {
                                                config.colors.line_numbers.normal = val
                                            }
                                        }
                                        "selected" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                value,
                                                &["colors", "line_numbers", "selected"],
                                                &config.colors.line_numbers.selected,
                                            )? {
                                                config.colors.line_numbers.selected = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["colors", "line_numbers", x],
                                                cell.span(),
                                            ))
                                        }
                                    }
                                }
                            }
                            x => return Err(invalid_field(&["colors", x], cell.span())),
                        }
                    }
                }
                "keybindings" => {
                    let cell = follow_cell_path(value, &["keybindings"]).unwrap();
                    let columns = match &cell {
                        Value::Record { val: rec, .. } => rec.columns().collect::<Vec<_>>(),
                        x => return Err(invalid_type(x, &["keybindings"], "record")),
                    };

                    for column in columns {
                        match column.as_str() {
                            "quit" => {
                                if let Some(val) = try_key(value, &["keybindings", "quit"])? {
                                    config.keybindings.quit = val
                                }
                            }
                            "insert" => {
                                if let Some(val) = try_key(value, &["keybindings", "insert"])? {
                                    config.keybindings.insert = val
                                }
                            }
                            "normal" => {
                                if let Some(val) = try_key(value, &["keybindings", "normal"])? {
                                    config.keybindings.normal = val
                                }
                            }
                            "navigation" => {
                                let cell = follow_cell_path(value, &["keybindings", "navigation"])
                                    .unwrap();
                                let columns = match &cell {
                                    Value::Record { val: rec, .. } => {
                                        rec.columns().collect::<Vec<_>>()
                                    }
                                    x => {
                                        return Err(invalid_type(
                                            x,
                                            &["keybindings", "navigation"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "up" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "up"],
                                            )? {
                                                config.keybindings.navigation.up = val
                                            }
                                        }
                                        "down" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "down"],
                                            )? {
                                                config.keybindings.navigation.down = val
                                            }
                                        }
                                        "left" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "left"],
                                            )? {
                                                config.keybindings.navigation.left = val
                                            }
                                        }
                                        "right" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "right"],
                                            )? {
                                                config.keybindings.navigation.right = val
                                            }
                                        }
                                        "half_page_up" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "half_page_up"],
                                            )? {
                                                config.keybindings.navigation.half_page_up = val
                                            }
                                        }
                                        "half_page_down" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "half_page_down"],
                                            )? {
                                                config.keybindings.navigation.half_page_down = val
                                            }
                                        }
                                        "goto_top" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "goto_top"],
                                            )? {
                                                config.keybindings.navigation.goto_top = val
                                            }
                                        }
                                        "goto_bottom" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "goto_bottom"],
                                            )? {
                                                config.keybindings.navigation.goto_bottom = val
                                            }
                                        }
                                        "goto_line" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "navigation", "goto_line"],
                                            )? {
                                                config.keybindings.navigation.goto_line = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["keybindings", "navigation", x],
                                                cell.span(),
                                            ));
                                        }
                                    }
                                }
                            }
                            "peek" => {
                                if let Some(val) = try_key(value, &["keybindings", "peek"])? {
                                    config.keybindings.peek = val
                                }
                            }
                            "peeking" => {
                                let cell =
                                    follow_cell_path(value, &["keybindings", "peeking"]).unwrap();
                                let columns = match &cell {
                                    Value::Record { val: rec, .. } => {
                                        rec.columns().collect::<Vec<_>>()
                                    }
                                    x => {
                                        return Err(invalid_type(
                                            x,
                                            &["keybindings", "peeking"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "all" => {
                                            if let Some(val) =
                                                try_key(value, &["keybindings", "peeking", "all"])?
                                            {
                                                config.keybindings.peeking.all = val
                                            }
                                        }
                                        "cell_path" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "peeking", "cell_path"],
                                            )? {
                                                config.keybindings.peeking.cell_path = val
                                            }
                                        }
                                        "under" => {
                                            if let Some(val) = try_key(
                                                value,
                                                &["keybindings", "peeking", "under"],
                                            )? {
                                                config.keybindings.peeking.under = val
                                            }
                                        }
                                        "view" => {
                                            if let Some(val) =
                                                try_key(value, &["keybindings", "peeking", "view"])?
                                            {
                                                config.keybindings.peeking.view = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["keybindings", "peeking", x],
                                                cell.span(),
                                            ));
                                        }
                                    }
                                }
                            }
                            "transpose" => {
                                if let Some(val) = try_key(value, &["keybindings", "tranpose"])? {
                                    config.keybindings.transpose = val
                                }
                            }
                            x => return Err(invalid_field(&["keybindings", x], cell.span())),
                        }
                    }
                }
                x => return Err(invalid_field(&[x], value.span())),
            }
        }

        Ok(config)
    }
}

// TODO: add proper assert error messages
#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use nu_protocol::{record, Record, Value};

    use crate::handler::repr_key;

    use super::Config;

    #[test]
    fn keycode_representation() {
        for (key, modifiers, expected) in [
            (KeyCode::Char('x'), KeyModifiers::NONE, "x"),
            (KeyCode::Char('x'), KeyModifiers::CONTROL, "<c-x>"),
            (KeyCode::Left, KeyModifiers::NONE, "←"),
            (KeyCode::Esc, KeyModifiers::NONE, "<esc>"),
            (KeyCode::Enter, KeyModifiers::NONE, "⏎"),
            (KeyCode::Home, KeyModifiers::NONE, "??"),
        ] {
            assert_eq!(repr_key(&KeyEvent::new(key, modifiers)), expected);
        }
    }

    #[test]
    fn parse_invalid_config() {
        assert_eq!(
            Config::from_value(&Value::test_string("x")),
            Ok(Config::default())
        );
    }

    #[test]
    fn parse_empty_config() {
        assert_eq!(
            Config::from_value(&Value::test_record(Record::new())),
            Ok(Config::default())
        );
    }

    #[test]
    fn parse_config_with_invalid_field() {
        let value = Value::test_record(record! {
            "x" => Value::test_nothing()
        });
        let result = Config::from_value(&value);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.labels[0].text.contains("not a valid config field"));

        let value = Value::test_record(record! {
            "colors" => Value::test_record(record! {
                "foo" => Value::test_nothing()
            })
        });
        let result = Config::from_value(&value);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.labels[0].text.contains("not a valid config field"));
    }

    #[test]
    fn parse_config() {
        let value = Value::test_record(record! {
            "show_cell_path" => Value::test_bool(true)
        });
        assert_eq!(Config::from_value(&value), Ok(Config::default()));

        let value = Value::test_record(record! {
            "show_cell_path" => Value::test_bool(false)
        });
        let expected = Config {
            show_cell_path: false,
            ..Default::default()
        };
        assert_eq!(Config::from_value(&value), Ok(expected));

        let value = Value::test_record(record! {
            "keybindings" => Value::test_record(record!{
                "navigation" => Value::test_record(record!{
                    "up" => Value::test_string("x")
                })
            }),
        });

        let mut expected = Config::default();
        expected.keybindings.navigation.up = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(Config::from_value(&value), Ok(expected));
    }

    #[test]
    fn same_as_default() {
        assert_eq!(
            Config::default(),
            Config::from_value(
                &nuon::from_nuon(include_str!("../../examples/config/default.nuon"), None).unwrap()
            )
            .unwrap()
        )
    }
}
