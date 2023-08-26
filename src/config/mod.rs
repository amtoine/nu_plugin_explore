//! management of the outside configuration of `explore`
//!
//! this module
//! 1. holds the data structure of the [`Config`]
//! 1. gives default values to a [`Config`] with [`Config::default`]
//! 1. parses a Nushell [`Value`](https://docs.rs/nu-protocol/0.83.1/nu_protocol/enum.Value.html) into a valid [`Config`]
use crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};

use nu_plugin::LabeledError;
use nu_protocol::Value;

mod parsing;
use parsing::{
    follow_cell_path, invalid_field, invalid_type, try_bool, try_fg_bg_colors, try_key, try_layout,
    try_modifier, try_string,
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
    pub up: KeyCode,
    /// go one row down in the data
    pub down: KeyCode,
    /// go one level higher in the data
    pub left: KeyCode,
    /// go one level deeper in the data
    pub right: KeyCode,
}

/// the bindings in PEEKING mode (see [crate::app::Mode::Peeking])
#[derive(Clone, PartialEq, Debug)]
pub struct PeekingBindingsMap {
    /// peek the whole data structure
    pub all: KeyCode,
    /// peek the current level
    pub current: KeyCode,
    /// peek the current level, but only the row under the cursor
    pub under: KeyCode,
}

/// the keybindings mapping
#[derive(Clone, PartialEq, Debug)]
pub struct KeyCodeBindingsMap {
    pub quit: KeyCode,
    /// go into INSERT mode (see [crate::app::Mode::Insert])
    pub insert: KeyCode,
    /// go back into NORMAL mode (see [crate::app::Mode::Normal])
    pub normal: KeyCode,
    pub navigation: NavigationBindingsMap,
    /// go into PEEKING mode (see [crate::app::Mode::Peeking])
    pub peek: KeyCode,
    pub peeking: PeekingBindingsMap,
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
    pub keybindings: KeyCodeBindingsMap,
    pub show_cell_path: bool,
    pub layout: Layout,
    pub show_table_header: bool,
}

impl Config {
    pub fn default() -> Self {
        // "reset" is used instead of "black" in a dark terminal because, when the terminal is actually
        // black, "black" is not really black which is ugly, whereas "reset" is really black.
        Self {
            show_cell_path: true,
            show_table_header: true,
            layout: Layout::Table,
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
                        background: Color::Reset,
                        foreground: Color::White,
                    },
                    insert: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::LightYellow,
                    },
                    peek: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::LightGreen,
                    },
                    bottom: BgFgColorConfig {
                        background: Color::Reset,
                        foreground: Color::LightMagenta,
                    },
                },
            },
            keybindings: KeyCodeBindingsMap {
                quit: KeyCode::Char('q'),
                insert: KeyCode::Char('i'),
                normal: KeyCode::Esc,
                navigation: NavigationBindingsMap {
                    left: KeyCode::Char('h'),
                    down: KeyCode::Char('j'),
                    up: KeyCode::Char('k'),
                    right: KeyCode::Char('l'),
                },
                peek: KeyCode::Char('p'),
                peeking: PeekingBindingsMap {
                    all: KeyCode::Char('a'),
                    current: KeyCode::Char('c'),
                    under: KeyCode::Char('p'),
                },
            },
        }
    }

    pub fn from_value(value: Value) -> Result<Self, LabeledError> {
        let mut config = Config::default();

        for column in value.columns() {
            match column.as_str() {
                "show_cell_path" => {
                    if let Some(val) = try_bool(&value, &["show_cell_path"])? {
                        config.show_cell_path = val
                    }
                }
                "show_table_header" => {
                    if let Some(val) = try_bool(&value, &["show_table_header"])? {
                        config.show_table_header = val
                    }
                }
                "layout" => {
                    if let Some(val) = try_layout(&value, &["layout"])? {
                        config.layout = val
                    }
                }
                "colors" => {
                    let (columns, span) = match follow_cell_path(&value, &["colors"]).unwrap() {
                        Value::Record { cols, span, .. } => (cols, span),
                        x => return Err(invalid_type(&x, &["colors"], "record")),
                    };

                    for column in columns {
                        match column.as_str() {
                            "normal" => {
                                let (columns, span) = match follow_cell_path(
                                    &value,
                                    &["colors", "normal"],
                                )
                                .unwrap()
                                {
                                    Value::Record { cols, span, .. } => (cols, span),
                                    x => {
                                        return Err(invalid_type(
                                            &x,
                                            &["colors", "normal"],
                                            "record",
                                        ))
                                    }
                                };

                                for column in columns {
                                    match column.as_str() {
                                        "name" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                &value,
                                                &["colors", "normal", "name"],
                                                &config.colors.normal.name,
                                            )? {
                                                config.colors.normal.name = val
                                            }
                                        }
                                        "data" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                &value,
                                                &["colors", "normal", "data"],
                                                &config.colors.normal.data,
                                            )? {
                                                config.colors.normal.data = val
                                            }
                                        }
                                        "shape" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                &value,
                                                &["colors", "normal", "shape"],
                                                &config.colors.normal.shape,
                                            )? {
                                                config.colors.normal.shape = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["colors", "normal", x],
                                                Some(span),
                                            ))
                                        }
                                    }
                                }
                            }
                            "selected" => {
                                if let Some(val) = try_fg_bg_colors(
                                    &value,
                                    &["colors", "selected"],
                                    &config.colors.selected,
                                )? {
                                    config.colors.selected = val
                                }
                            }
                            "selected_symbol" => {
                                if let Some(val) =
                                    try_string(&value, &["colors", "selected_symbol"])?
                                {
                                    config.colors.selected_symbol = val
                                }
                            }
                            "selected_modifier" => {
                                if let Some(val) =
                                    try_modifier(&value, &["colors", "selected_modifier"])?
                                {
                                    config.colors.selected_modifier = val
                                }
                            }
                            "status_bar" => {
                                let (columns, span) =
                                    match follow_cell_path(&value, &["colors", "status_bar"])
                                        .unwrap()
                                    {
                                        Value::Record { cols, span, .. } => (cols, span),
                                        x => {
                                            return Err(invalid_type(
                                                &x,
                                                &["colors", "status_bar"],
                                                "record",
                                            ))
                                        }
                                    };

                                for column in columns {
                                    match column.as_str() {
                                        "normal" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                &value,
                                                &["colors", "status_bar", "normal"],
                                                &config.colors.status_bar.normal,
                                            )? {
                                                config.colors.status_bar.normal = val
                                            }
                                        }
                                        "insert" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                &value,
                                                &["colors", "status_bar", "insert"],
                                                &config.colors.status_bar.insert,
                                            )? {
                                                config.colors.status_bar.insert = val
                                            }
                                        }
                                        "peek" => {
                                            if let Some(val) = try_fg_bg_colors(
                                                &value,
                                                &["colors", "status_bar", "peek"],
                                                &config.colors.status_bar.peek,
                                            )? {
                                                config.colors.status_bar.peek = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["colors", "status_bar", x],
                                                Some(span),
                                            ))
                                        }
                                    }
                                }
                            }
                            x => return Err(invalid_field(&["colors", x], Some(span))),
                        }
                    }
                }
                "keybindings" => {
                    let (columns, span) = match follow_cell_path(&value, &["keybindings"]).unwrap()
                    {
                        Value::Record { cols, span, .. } => (cols, span),
                        x => return Err(invalid_type(&x, &["keybindings"], "record")),
                    };

                    for column in columns {
                        match column.as_str() {
                            "quit" => {
                                if let Some(val) = try_key(&value, &["keybindings", "quit"])? {
                                    config.keybindings.quit = val
                                }
                            }
                            "insert" => {
                                if let Some(val) = try_key(&value, &["keybindings", "insert"])? {
                                    config.keybindings.insert = val
                                }
                            }
                            "normal" => {
                                if let Some(val) = try_key(&value, &["keybindings", "normal"])? {
                                    config.keybindings.normal = val
                                }
                            }
                            "navigation" => {
                                let (columns, span) =
                                    match follow_cell_path(&value, &["keybindings", "navigation"])
                                        .unwrap()
                                    {
                                        Value::Record { cols, span, .. } => (cols, span),
                                        x => {
                                            return Err(invalid_type(
                                                &x,
                                                &["keybindings", "navigation"],
                                                "record",
                                            ))
                                        }
                                    };

                                for column in columns {
                                    match column.as_str() {
                                        "up" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "navigation", "up"],
                                            )? {
                                                config.keybindings.navigation.up = val
                                            }
                                        }
                                        "down" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "navigation", "down"],
                                            )? {
                                                config.keybindings.navigation.down = val
                                            }
                                        }
                                        "left" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "navigation", "left"],
                                            )? {
                                                config.keybindings.navigation.left = val
                                            }
                                        }
                                        "right" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "navigation", "right"],
                                            )? {
                                                config.keybindings.navigation.right = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["keybindings", "navigation", x],
                                                Some(span),
                                            ));
                                        }
                                    }
                                }
                            }
                            "peek" => {
                                if let Some(val) = try_key(&value, &["keybindings", "peek"])? {
                                    config.keybindings.peek = val
                                }
                            }
                            "peeking" => {
                                let (columns, span) =
                                    match follow_cell_path(&value, &["keybindings", "peeking"])
                                        .unwrap()
                                    {
                                        Value::Record { cols, span, .. } => (cols, span),
                                        x => {
                                            return Err(invalid_type(
                                                &x,
                                                &["keybindings", "peeking"],
                                                "record",
                                            ))
                                        }
                                    };

                                for column in columns {
                                    match column.as_str() {
                                        "all" => {
                                            if let Some(val) =
                                                try_key(&value, &["keybindings", "peeking", "all"])?
                                            {
                                                config.keybindings.peeking.all = val
                                            }
                                        }
                                        "current" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "peeking", "current"],
                                            )? {
                                                config.keybindings.peeking.current = val
                                            }
                                        }
                                        "under" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "peeking", "under"],
                                            )? {
                                                config.keybindings.peeking.under = val
                                            }
                                        }
                                        x => {
                                            return Err(invalid_field(
                                                &["keybindings", "peeking", x],
                                                Some(span),
                                            ));
                                        }
                                    }
                                }
                            }
                            x => return Err(invalid_field(&["keybindings", x], Some(span))),
                        }
                    }
                }
                x => return Err(invalid_field(&[x], value.span().ok())),
            }
        }

        Ok(config)
    }
}

/// represent a [`KeyCode`] as a simple string
pub fn repr_keycode(keycode: &KeyCode) -> String {
    match keycode {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Left => "←".into(),
        KeyCode::Up => "↑".into(),
        KeyCode::Right => "→".into(),
        KeyCode::Down => "↓".into(),
        KeyCode::Esc => "<esc>".into(),
        _ => "??".into(),
    }
}

// TODO: add proper assert error messages
#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use nu_protocol::Value;

    use super::{repr_keycode, Config};

    #[test]
    fn keycode_representation() {
        assert_eq!(repr_keycode(&KeyCode::Char('x')), "x".to_string());
        assert_eq!(repr_keycode(&KeyCode::Left), "←".to_string());
        assert_eq!(repr_keycode(&KeyCode::Esc), "<esc>".to_string());
        assert_eq!(repr_keycode(&KeyCode::Enter), "??".to_string());
    }

    #[test]
    fn parse_invalid_config() {
        assert_eq!(
            Config::from_value(Value::test_string("x")),
            Ok(Config::default())
        );
    }

    #[test]
    fn parse_empty_config() {
        let cols: Vec<&str> = vec![];
        assert_eq!(
            Config::from_value(Value::test_record(cols, vec![])),
            Ok(Config::default())
        );
    }

    #[test]
    fn parse_config_with_invalid_field() {
        let value = Value::test_record(vec!["x"], vec![Value::test_nothing()]);
        let result = Config::from_value(value);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.msg.contains("not a valid config field"));

        let value = Value::test_record(
            vec!["colors"],
            vec![Value::test_record(vec!["foo"], vec![Value::test_nothing()])],
        );
        let result = Config::from_value(value);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.msg.contains("not a valid config field"));
    }

    #[test]
    fn parse_config() {
        let value = Value::test_record(vec!["show_cell_path"], vec![Value::test_bool(true)]);
        assert_eq!(Config::from_value(value), Ok(Config::default()));

        let value = Value::test_record(vec!["show_cell_path"], vec![Value::test_bool(false)]);
        let mut expected = Config::default();
        expected.show_cell_path = false;
        assert_eq!(Config::from_value(value), Ok(expected));

        let value = Value::test_record(
            vec!["keybindings"],
            vec![Value::test_record(
                vec!["navigation"],
                vec![Value::test_record(
                    vec!["up"],
                    vec![Value::test_string("x")],
                )],
            )],
        );
        let mut expected = Config::default();
        expected.keybindings.navigation.up = KeyCode::Char('x');
        assert_eq!(Config::from_value(value), Ok(expected));
    }
}
