use console::Key;
use ratatui::style::{Color, Modifier};

use nu_plugin::LabeledError;
use nu_protocol::{ast::PathMember, Span, Value};

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

fn invalid_field(cell_path: &[&str], span: Option<Span>) -> LabeledError {
    LabeledError {
        label: "invalid config".into(),
        msg: format!("`$.{}` is not a valid config field", cell_path.join("."),),
        span,
    }
}

fn invalid_type(value: &Value, cell_path: &[&str], expected: &str) -> LabeledError {
    LabeledError {
        label: "invalid config".into(),
        msg: format!(
            "`$.{}` should be a {expected}, found {}",
            cell_path.join("."),
            value.get_type().to_string()
        ),
        span: value.span().ok(),
    }
}

fn try_bool(value: &Value, cell_path: &[&str]) -> Result<Option<bool>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Bool { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "bool")),
        _ => Ok(None),
    }
}

fn try_string(value: &Value, cell_path: &[&str]) -> Result<Option<String>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

fn try_modifier(value: &Value, cell_path: &[&str]) -> Result<Option<Modifier>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Nothing { .. }) => Ok(Some(Modifier::empty())),
        Some(Value::String { val, .. }) => match val.as_str() {
            "" => Ok(Some(Modifier::empty())),
            "bold" => Ok(Some(Modifier::BOLD)),
            "italic" => Ok(Some(Modifier::ITALIC)),
            "underline" => Ok(Some(Modifier::UNDERLINED)),
            "blink" => Ok(Some(Modifier::SLOW_BLINK)),
            x => {
                return Err(LabeledError {
                    label: "invalid config".into(),
                    msg: format!(
                        r#"`$.{}` should be the empty string, one of [italic, bold, underline, blink] or null, found {}"#,
                        cell_path.join("."),
                        x
                    ),
                    span: value.span().ok(),
                })
            }
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string or null")),
        _ => Ok(None),
    }
}

fn try_color(value: &Value, cell_path: &[&str]) -> Result<Option<Color>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => match val.as_str() {
            "black" => Ok(Some(Color::Black)),
            "red" => Ok(Some(Color::Red)),
            "green" => Ok(Some(Color::Green)),
            "yellow" => Ok(Some(Color::Yellow)),
            "blue" => Ok(Some(Color::Blue)),
            "magenta" => Ok(Some(Color::Magenta)),
            "cyan" => Ok(Some(Color::Cyan)),
            "gray" => Ok(Some(Color::Gray)),
            "darkgray" => Ok(Some(Color::DarkGray)),
            "lightred" => Ok(Some(Color::LightRed)),
            "lightgreen" => Ok(Some(Color::LightGreen)),
            "lightyellow" => Ok(Some(Color::LightYellow)),
            "lightblue" => Ok(Some(Color::LightBlue)),
            "lightmagenta" => Ok(Some(Color::LightMagenta)),
            "lightcyan" => Ok(Some(Color::LightCyan)),
            "white" => Ok(Some(Color::White)),
            x => {
                return Err(LabeledError {
                    label: "invalid config".into(),
                    msg: format!(
                        r#"`$.{}` should be one of [black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white] , found {}"#,
                        cell_path.join("."),
                        x
                    ),
                    span: value.span().ok(),
                })
            }
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

fn try_fg_bg_colors(
    value: &Value,
    cell_path: &[&str],
    default: &BgFgColorConfig,
) -> Result<Option<BgFgColorConfig>, LabeledError> {
    let (columns, span) = match follow_cell_path(&value, &cell_path).unwrap() {
        Value::Record { cols, span, .. } => (cols, span),
        x => return Err(invalid_type(&x, &cell_path, "record")),
    };

    let mut colors: BgFgColorConfig = default.clone();

    for column in columns {
        match column.as_str() {
            "background" => {
                let mut cell_path = cell_path.to_vec();
                cell_path.push("background");
                if let Some(val) = try_color(&value, &cell_path)? {
                    colors.background = val
                }
            }
            "foreground" => {
                let mut cell_path = cell_path.to_vec();
                cell_path.push("foreground");
                if let Some(val) = try_color(&value, &cell_path)? {
                    colors.foreground = val
                }
            }
            x => {
                let mut cell_path = cell_path.to_vec();
                cell_path.push(x);
                return Err(invalid_field(&cell_path, Some(span)));
            }
        }
    }

    Ok(Some(colors))
}

fn try_key(value: &Value, cell_path: &[&str]) -> Result<Option<Key>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => match val.as_str() {
            "up" => Ok(Some(Key::ArrowUp)),
            "down" => Ok(Some(Key::ArrowDown)),
            "left" => Ok(Some(Key::ArrowLeft)),
            "right" => Ok(Some(Key::ArrowRight)),
            "escape" => Ok(Some(Key::Escape)),
            x => {
                if x.len() != 1 {
                    return Err(LabeledError {
                        label: "invalid config".into(),
                        msg: format!(
                            r#"`$.{}` should be a character or one of [up, down, left, right, escape] , found {}"#,
                            cell_path.join("."),
                            x
                        ),
                        span: value.span().ok(),
                    });
                }

                Ok(Some(Key::Char(x.to_string().chars().nth(0).unwrap())))
            }
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

fn follow_cell_path(value: &Value, cell_path: &[&str]) -> Option<Value> {
    let cell_path = cell_path
        .iter()
        .map(|cp| PathMember::String {
            val: cp.to_string(),
            span: Span::unknown(),
            optional: false,
        })
        .collect::<Vec<PathMember>>();

    value.clone().follow_cell_path(&cell_path, false).ok()
}
