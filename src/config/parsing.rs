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

fn u8_out_of_range(value: i64, cell_path: &[&str], span: Option<Span>) -> LabeledError {
    LabeledError {
        label: "invalid config".into(),
        msg: format!(
            "`$.{}` should be an integer between 0 and 255, found {}",
            cell_path.join("."),
            value
        ),
        span,
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
                    r#"`$.{}` should be a u8, a list of three u8s or one of [black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white] , found {}"#,
                    cell_path.join("."),
                    x
                ),
                span: value.span().ok(),
            }),
        },
        Some(Value::Int { val, .. }) => {
            if !(0..=255).contains(&val) {
                // FIXME: use a real span?
                return Err(u8_out_of_range(val, cell_path, None));
            }

            Ok(Some(Color::Rgb(val as u8, val as u8, val as u8)))
        }
        Some(Value::List { vals, .. }) => {
            if vals.len() != 3 {
                return Err(LabeledError {
                    label: "invalid config".into(),
                    msg: format!("`$.{}` is not a valid config field, expected a list of three u8, found {} items", cell_path.join("."), vals.len()),
                    // FIXME: use a real span?
                    span: None,
                });
            }

            let mut channels: Vec<u8> = vec![];

            for (i, val) in vals.iter().enumerate() {
                let mut cell_path = cell_path.to_vec().clone();

                let tail = format!("{}", i);
                cell_path.push(&tail);

                match val {
                    Value::Int { val: x, .. } => {
                        if (*x < 0) | (*x > 255) {
                            return Err(u8_out_of_range(*x, &cell_path, val.span().ok()));
                        }

                        channels.push(*x as u8);
                    }
                    x => {
                        return Err(invalid_type(x, &cell_path, "u8"));
                    }
                }
            }

            Ok(Some(Color::Rgb(channels[0], channels[1], channels[2])))
        }
        Some(x) => Err(invalid_type(&x, cell_path, "string, u8 or [u8, u8, u8]")),
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

// TODO: add proper assert error messages
#[cfg(test)]
mod tests {
    use console::Key;
    use nu_plugin::LabeledError;
    use nu_protocol::Value;
    use ratatui::style::{Color, Modifier};

    use super::{
        follow_cell_path, try_bool, try_color, try_fg_bg_colors, try_key, try_layout, try_modifier,
        try_string,
    };
    use crate::config::{BgFgColorConfig, Layout};

    #[test]
    fn follow_str_cell_path() {
        let inner_record_a = Value::test_int(1);
        let inner_record_b = Value::test_int(2);
        let record = Value::test_record(
            vec!["a", "b"],
            vec![inner_record_a.clone(), inner_record_b.clone()],
        );
        let string = Value::test_string("some string");
        let int = Value::test_int(123);

        let value = Value::test_record(
            vec!["r", "s", "i"],
            vec![record.clone(), string.clone(), int.clone()],
        );

        assert_eq!(follow_cell_path(&value, &[]), Some(value.clone()));
        assert_eq!(follow_cell_path(&value, &["r"]), Some(record));
        assert_eq!(follow_cell_path(&value, &["s"]), Some(string));
        assert_eq!(follow_cell_path(&value, &["i"]), Some(int));
        assert_eq!(follow_cell_path(&value, &["x"]), None);
        assert_eq!(follow_cell_path(&value, &["r", "a"]), Some(inner_record_a));
        assert_eq!(follow_cell_path(&value, &["r", "b"]), Some(inner_record_b));
        assert_eq!(follow_cell_path(&value, &["r", "x"]), None);
    }

    fn test_tried_error<T>(
        result: Result<Option<T>, LabeledError>,
        cell_path: &str,
        expected: &str,
    ) {
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.label, "invalid config");
        assert_eq!(err.msg, format!("`$.{}` {}", cell_path, expected));
    }

    #[test]
    fn trying_bool() {
        test_tried_error(
            try_bool(&Value::test_string("not a bool"), &[]),
            "",
            "should be a bool, found string",
        );
        test_tried_error(
            try_bool(&Value::test_int(123), &[]),
            "",
            "should be a bool, found int",
        );

        assert_eq!(try_bool(&Value::test_bool(true), &[]), Ok(Some(true)));
        assert_eq!(try_bool(&Value::test_bool(false), &[]), Ok(Some(false)));
        assert_eq!(try_bool(&Value::test_nothing(), &["x"]), Ok(None));
    }

    #[test]
    fn trying_string() {
        test_tried_error(
            try_string(&Value::test_bool(true), &[]),
            "",
            "should be a string, found bool",
        );
        test_tried_error(
            try_string(&Value::test_int(123), &[]),
            "",
            "should be a string, found int",
        );

        assert_eq!(
            try_string(&Value::test_string("my string"), &[]),
            Ok(Some("my string".to_string()))
        );
        assert_eq!(
            try_string(&Value::test_string("my string"), &["x"]),
            Ok(None)
        );
    }

    #[test]
    fn trying_key() {
        test_tried_error(
            try_key(&Value::test_bool(true), &[]),
            "",
            "should be a string, found bool",
        );
        test_tried_error(
            try_key(&Value::test_int(123), &[]),
            "",
            "should be a string, found int",
        );
        test_tried_error(
            try_key(&Value::test_string("enter"), &[]),
            "",
            "should be a character or one of [up, down, left, right, escape] , found enter",
        );

        let cases = vec![
            ("up", Key::ArrowUp),
            ("down", Key::ArrowDown),
            ("left", Key::ArrowLeft),
            ("right", Key::ArrowRight),
            ("escape", Key::Escape),
            ("a", Key::Char('a')),
            ("b", Key::Char('b')),
            ("x", Key::Char('x')),
        ];

        for (input, expected) in cases {
            assert_eq!(try_key(&Value::test_string(input), &[]), Ok(Some(expected)));
        }
    }

    #[test]
    fn trying_layout() {
        test_tried_error(
            try_layout(&Value::test_bool(true), &[]),
            "",
            "should be a string, found bool",
        );
        test_tried_error(
            try_layout(&Value::test_int(123), &[]),
            "",
            "should be a string, found int",
        );
        test_tried_error(
            try_layout(&Value::test_string("collapsed"), &[]),
            "",
            "should be one of [table, compact] , found collapsed",
        );

        let cases = vec![("table", Layout::Table), ("compact", Layout::Compact)];

        for (input, expected) in cases {
            assert_eq!(
                try_layout(&Value::test_string(input), &[]),
                Ok(Some(expected))
            );
        }
    }

    #[test]
    fn trying_modifier() {
        test_tried_error(
            try_modifier(&Value::test_bool(true), &[]),
            "",
            "should be a string or null, found bool",
        );
        test_tried_error(
            try_modifier(&Value::test_int(123), &[]),
            "",
            "should be a string or null, found int",
        );
        test_tried_error(
            try_modifier(&Value::test_string("x"), &[]),
            "",
            "should be the empty string, one of [italic, bold, underline, blink] or null, found x",
        );

        assert_eq!(
            try_modifier(&Value::test_nothing(), &[]),
            Ok(Some(Modifier::empty()))
        );

        let cases = vec![
            ("", Modifier::empty()),
            ("italic", Modifier::ITALIC),
            ("bold", Modifier::BOLD),
            ("underline", Modifier::UNDERLINED),
            ("blink", Modifier::SLOW_BLINK),
        ];

        for (input, expected) in cases {
            assert_eq!(
                try_modifier(&Value::test_string(input), &[]),
                Ok(Some(expected))
            );
        }
    }

    #[test]
    fn trying_color() {
        test_tried_error(
            try_color(&Value::test_bool(true), &[]),
            "",
            "should be a string, found bool",
        );
        test_tried_error(
            try_color(&Value::test_int(123), &[]),
            "",
            "should be a string, found int",
        );
        test_tried_error(
            try_color(&Value::test_string("x"), &[]),
            "",
            "should be one of [black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white] , found x",
        );

        let cases = vec![
            ("black", Color::Black),
            ("red", Color::Red),
            ("green", Color::Green),
            ("blue", Color::Blue),
        ];

        for (input, expected) in cases {
            assert_eq!(
                try_color(&Value::test_string(input), &[]),
                Ok(Some(expected))
            );
        }
    }

    #[test]
    fn trying_fg_bg_colors() {
        let default_color = BgFgColorConfig {
            background: Color::Reset,
            foreground: Color::Reset,
        };

        test_tried_error(
            try_fg_bg_colors(&Value::test_bool(true), &[], &default_color),
            "",
            "should be a record, found bool",
        );
        test_tried_error(
            try_fg_bg_colors(&Value::test_int(123), &[], &default_color),
            "",
            "should be a record, found int",
        );
        test_tried_error(
            try_fg_bg_colors(&Value::test_string("x"), &[], &default_color),
            "",
            "should be a record, found string",
        );
        test_tried_error(
            try_fg_bg_colors(
                &Value::test_record(vec!["x"], vec![Value::test_nothing()]),
                &[],
                &default_color,
            ),
            "x",
            "is not a valid config field",
        );

        let cases = vec![
            (vec![], vec![], default_color.clone()),
            (
                vec!["foreground"],
                vec![Value::test_string("green")],
                BgFgColorConfig {
                    foreground: Color::Green,
                    background: Color::Reset,
                },
            ),
            (
                vec!["background"],
                vec![Value::test_string("blue")],
                BgFgColorConfig {
                    foreground: Color::Reset,
                    background: Color::Blue,
                },
            ),
            (
                vec!["foreground", "background"],
                vec![Value::test_string("green"), Value::test_string("blue")],
                BgFgColorConfig {
                    foreground: Color::Green,
                    background: Color::Blue,
                },
            ),
        ];

        for (cols, vals, expected) in cases {
            assert_eq!(
                try_fg_bg_colors(&Value::test_record(cols, vals), &[], &default_color),
                Ok(Some(expected))
            );
        }
    }
}
