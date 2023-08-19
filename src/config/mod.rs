//! management of the outside configuration of `explore`
//!
//! this module
//! 1. holds the data structure of the [`Config`]
//! 1. gives default values to a [`Config`] with [`Config::default`]
//! 1. parses a Nushell [`Value`](https://docs.rs/nu-protocol/0.83.1/nu_protocol/enum.Value.html) into a valid [`Config`]
use console::Key;
use ratatui::style::{Color, Modifier};

use nu_plugin::LabeledError;
use nu_protocol::Value;

mod parsing;
use parsing::{
    follow_cell_path, invalid_field, invalid_type, try_bool, try_fg_bg_colors, try_key, try_layout,
    try_modifier, try_string,
};

/// the configuration for the status bar colors in all [`crate::app::Mode`]s
pub(super) struct StatusBarColorConfig {
    pub normal: BgFgColorConfig,
    pub insert: BgFgColorConfig,
    pub peek: BgFgColorConfig,
}

/// the colors of the application
pub(super) struct ColorConfig {
    /// the color when a row is NOT selected
    pub normal: BgFgColorConfig,
    /// the color when a row is selected
    pub selected: BgFgColorConfig,
    /// the modifier to apply to the row under the cursor
    pub selected_modifier: Modifier,
    /// the symbol to show to the left of the selected row under the cursor
    pub selected_symbol: String,
    pub status_bar: StatusBarColorConfig,
}

/// a pair of background / foreground colors
#[derive(Clone)]
pub(super) struct BgFgColorConfig {
    pub background: Color,
    pub foreground: Color,
}

/// the bindings in NORMAL mode (see [crate::app::Mode::Normal])
pub(super) struct NavigationBindingsMap {
    /// go one row up in the data
    pub up: Key,
    /// go one row down in the data
    pub down: Key,
    /// go one level higher in the data
    pub left: Key,
    /// go one level deeper in the data
    pub right: Key,
}

/// the bindings in PEEKING mode (see [crate::app::Mode::Peeking])
pub(super) struct PeekingBindingsMap {
    /// peek the whole data structure
    pub all: Key,
    /// peek the current level
    pub current: Key,
    /// peek the current level, but only the row under the cursor
    pub under: Key,
    pub quit: Key,
}

/// the keybindings mapping
pub(super) struct KeyBindingsMap {
    pub quit: Key,
    /// go into INSERT mode (see [crate::app::Mode::Insert])
    pub insert: Key,
    /// go back into NORMAL mode (see [crate::app::Mode::Normal])
    pub normal: Key,
    pub navigation: NavigationBindingsMap,
    /// go into PEEKING mode (see [crate::app::Mode::Peeking])
    pub peek: Key,
    pub peeking: PeekingBindingsMap,
}

/// the layout of the application
pub(super) enum Layout {
    /// show each row in a `[name, data, type]` column
    Table,
    /// show each row in compact form, to the left, `"{name}: ({type}) {data}"`
    Compact,
}

/// the configuration of the whole application
pub(super) struct Config {
    pub colors: ColorConfig,
    pub keybindings: KeyBindingsMap,
    pub show_cell_path: bool,
    pub layout: Layout,
}

impl Config {
    pub(super) fn default() -> Self {
        Self {
            show_cell_path: true,
            layout: Layout::Table,
            colors: ColorConfig {
                normal: BgFgColorConfig {
                    background: Color::Reset, // "Black" is not pure *black*
                    foreground: Color::White,
                },
                selected: BgFgColorConfig {
                    background: Color::White,
                    foreground: Color::Black,
                },
                selected_modifier: Modifier::BOLD,
                selected_symbol: "".into(),
                status_bar: StatusBarColorConfig {
                    normal: BgFgColorConfig {
                        background: Color::White,
                        foreground: Color::Black,
                    },
                    insert: BgFgColorConfig {
                        background: Color::LightYellow,
                        foreground: Color::Black,
                    },
                    peek: BgFgColorConfig {
                        background: Color::LightGreen,
                        foreground: Color::Black,
                    },
                },
            },
            keybindings: KeyBindingsMap {
                quit: Key::Char('q'),
                insert: Key::Char('i'),
                normal: Key::Escape,
                navigation: NavigationBindingsMap {
                    left: Key::Char('h'),
                    down: Key::Char('j'),
                    up: Key::Char('k'),
                    right: Key::Char('l'),
                },
                peek: Key::Char('p'),
                peeking: PeekingBindingsMap {
                    all: Key::Char('a'),
                    current: Key::Char('c'),
                    under: Key::Char('u'),
                    quit: Key::Escape,
                },
            },
        }
    }

    pub(super) fn from_value(value: Value) -> Result<Self, LabeledError> {
        let mut config = Config::default();

        for column in value.columns() {
            match column.as_str() {
                "show_cell_path" => {
                    if let Some(val) = try_bool(&value, &["show_cell_path"])? {
                        config.show_cell_path = val
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
                                if let Some(val) = try_fg_bg_colors(
                                    &value,
                                    &["colors", "normal"],
                                    &config.colors.normal,
                                )? {
                                    config.colors.normal = val
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
                                        "quit" => {
                                            if let Some(val) = try_key(
                                                &value,
                                                &["keybindings", "peeking", "quit"],
                                            )? {
                                                config.keybindings.peeking.quit = val
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

/// represent a [`Key`] as a simple string
pub(super) fn repr_keycode(keycode: &Key) -> String {
    match keycode {
        Key::Char(c) => c.to_string(),
        Key::ArrowLeft => "←".into(),
        Key::ArrowUp => "↑".into(),
        Key::ArrowRight => "→".into(),
        Key::ArrowDown => "↓".into(),
        Key::Escape => "<esc>".into(),
        _ => "??".into(),
    }
}
