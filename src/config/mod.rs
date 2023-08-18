use console::Key;
use ratatui::style::{Color, Modifier};

use nu_plugin::LabeledError;
use nu_protocol::Value;

mod parsing;
use parsing::{
    follow_cell_path, invalid_field, invalid_type, try_bool, try_fg_bg_colors, try_key,
    try_modifier, try_string,
};

pub(super) struct ColorConfig {
    pub normal: BgFgColorConfig,
    pub selected: BgFgColorConfig,
    pub selected_modifier: Modifier,
    pub selected_symbol: String,
    pub status_bar: BgFgColorConfig,
}

#[derive(Clone)]
pub(super) struct BgFgColorConfig {
    pub background: Color,
    pub foreground: Color,
}

pub(super) struct NavigationBindingsMap {
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,
}
pub(super) struct PeekingBindingsMap {
    pub all: Key,
    pub current: Key,
    pub under: Key,
    pub quit: Key,
}

pub(super) struct KeyBindingsMap {
    pub quit: Key,
    pub insert: Key,
    pub normal: Key,
    pub navigation: NavigationBindingsMap,
    pub peek: Key,
    pub peeking: PeekingBindingsMap,
}

pub(super) struct Config {
    pub colors: ColorConfig,
    pub keybindings: KeyBindingsMap,
    pub show_cell_path: bool,
}

impl Config {
    pub(super) fn default() -> Self {
        Self {
            show_cell_path: true,
            colors: ColorConfig {
                normal: BgFgColorConfig {
                    background: Color::Black,
                    foreground: Color::White,
                },
                selected: BgFgColorConfig {
                    background: Color::White,
                    foreground: Color::Black,
                },
                selected_modifier: Modifier::BOLD,
                selected_symbol: "".into(),
                status_bar: BgFgColorConfig {
                    background: Color::White,
                    foreground: Color::Black,
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
                                if let Some(val) = try_fg_bg_colors(
                                    &value,
                                    &["colors", "status_bar"],
                                    &config.colors.status_bar,
                                )? {
                                    config.colors.status_bar = val
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
