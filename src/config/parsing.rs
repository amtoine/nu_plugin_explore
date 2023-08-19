//! utilities to parse a [`Value`](https://docs.rs/nu-protocol/0.83.1/nu_protocol/enum.Value.html)
//! into a configuration
use console::Key;
use ratatui::style::{Color, Modifier};

use nu_plugin::LabeledError;
use nu_protocol::{ast::PathMember, Span, Value};

use super::{BgFgColorConfig, Layout};

/// return an *invalid field* error
///
/// # Example
/// ```text
/// invalid_field(&["foo"], Some(span))
/// ```
/// would give an error like
/// ```nushell
/// Error:   × invalid config
///    ╭─[entry #3:1:1]
///  1 │ explore {foo: 123}
///    ·         ─────┬────
///    ·              ╰── `$.foo` is not a valid config field
///    ╰────
/// ```
pub(super) fn invalid_field(cell_path: &[&str], span: Option<Span>) -> LabeledError {
    LabeledError {
        label: "invalid config".into(),
        msg: format!("`$.{}` is not a valid config field", cell_path.join("."),),
        span,
    }
}

/// return an *invalid type* error
///
/// # Example
/// ```text
/// invalid_type(&some_int, &["layout"], "string"),
/// ```
/// would give an error like
/// ```nushell
/// Error:   × invalid config
///    ╭─[entry #7:1:1]
///  1 │ explore {layout: 123}
///    ·                  ─┬─
///    ·                   ╰── `$.layout` should be a string, found int
///    ╰────
/// ```
pub(super) fn invalid_type(value: &Value, cell_path: &[&str], expected: &str) -> LabeledError {
    LabeledError {
        label: "invalid config".into(),
        msg: format!(
            "`$.{}` should be a {expected}, found {}",
            cell_path.join("."),
            value.get_type()
        ),
        span: value.span().ok(),
    }
}

/// try to parse a bool in the *value* at the given *cell path*
pub(super) fn try_bool(value: &Value, cell_path: &[&str]) -> Result<Option<bool>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Bool { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "bool")),
        _ => Ok(None),
    }
}

/// try to parse a string in the *value* at the given *cell path*
pub(super) fn try_string(
    value: &Value,
    cell_path: &[&str],
) -> Result<Option<String>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

/// try to parse an ANSI modifier in the *value* at the given *cell path*
pub(super) fn try_modifier(
    value: &Value,
    cell_path: &[&str],
) -> Result<Option<Modifier>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Nothing { .. }) => Ok(Some(Modifier::empty())),
        Some(Value::String { val, .. }) => match val.as_str() {
            "" => Ok(Some(Modifier::empty())),
            "bold" => Ok(Some(Modifier::BOLD)),
            "italic" => Ok(Some(Modifier::ITALIC)),
            "underline" => Ok(Some(Modifier::UNDERLINED)),
            "blink" => Ok(Some(Modifier::SLOW_BLINK)),
            x => Err(LabeledError {
                label: "invalid config".into(),
                msg: format!(
                    r#"`$.{}` should be the empty string, one of [italic, bold, underline, blink] or null, found {}"#,
                    cell_path.join("."),
                    x
                ),
                span: value.span().ok(),
            }),
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string or null")),
        _ => Ok(None),
    }
}

/// try to parse a color in the *value* at the given *cell path*
pub(super) fn try_color(value: &Value, cell_path: &[&str]) -> Result<Option<Color>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => match val.as_str() {
            "reset" => Ok(Some(Color::Reset)),
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
            x => Err(LabeledError {
                label: "invalid config".into(),
                msg: format!(
                    r#"`$.{}` should be one of [black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white] , found {}"#,
                    cell_path.join("."),
                    x
                ),
                span: value.span().ok(),
            }),
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

/// try to parse a background / foreground color pair in the *value* at the given *cell path*
pub(super) fn try_fg_bg_colors(
    value: &Value,
    cell_path: &[&str],
    default: &BgFgColorConfig,
) -> Result<Option<BgFgColorConfig>, LabeledError> {
    let (columns, span) = match follow_cell_path(value, cell_path).unwrap() {
        Value::Record { cols, span, .. } => (cols, span),
        x => return Err(invalid_type(&x, cell_path, "record")),
    };

    let mut colors: BgFgColorConfig = default.clone();

    for column in columns {
        match column.as_str() {
            "background" => {
                let mut cell_path = cell_path.to_vec();
                cell_path.push("background");
                if let Some(val) = try_color(value, &cell_path)? {
                    colors.background = val
                }
            }
            "foreground" => {
                let mut cell_path = cell_path.to_vec();
                cell_path.push("foreground");
                if let Some(val) = try_color(value, &cell_path)? {
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

/// try to parse a key in the *value* at the given *cell path*
pub(super) fn try_key(value: &Value, cell_path: &[&str]) -> Result<Option<Key>, LabeledError> {
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

                #[allow(clippy::iter_nth_zero)]
                Ok(Some(Key::Char(x.to_string().chars().nth(0).unwrap())))
            }
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

/// try to parse a layout in the *value* at the given *cell path*
pub(super) fn try_layout(
    value: &Value,
    cell_path: &[&str],
) -> Result<Option<Layout>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => match val.as_str() {
            "table" => Ok(Some(Layout::Table)),
            "compact" => Ok(Some(Layout::Compact)),
            x => Err(LabeledError {
                label: "invalid config".into(),
                msg: format!(
                    r#"`$.{}` should be one of [table, compact] , found {}"#,
                    cell_path.join("."),
                    x
                ),
                span: value.span().ok(),
            }),
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

/// follow a cell path into a Value, giving the resulting Value if it exists
///
/// # Example
/// ```text
/// follow_cell_path(&value, &["foo", "bar", "baz"]).unwrap()
/// ```
/// would give `123` in a Nushell structure such as
/// ```nushell
/// {
///     foo: {
///         bar: {
///             baz: 123
///         }
///     }
/// }
/// ```
pub(super) fn follow_cell_path(value: &Value, cell_path: &[&str]) -> Option<Value> {
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
