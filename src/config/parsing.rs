//! utilities to parse a [`Value`](https://docs.rs/nu-protocol/0.83.1/nu_protocol/enum.Value.html)
//! into a configuration
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier};

use nu_protocol::LabeledError;
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
pub fn invalid_field(cell_path: &[&str], span: Span) -> LabeledError {
    LabeledError::new("invalid config").with_label(
        format!("`$.{}` is not a valid config field", cell_path.join("."),),
        span,
    )
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
pub fn invalid_type(value: &Value, cell_path: &[&str], expected: &str) -> LabeledError {
    LabeledError::new("invalid config").with_label(
        format!(
            "`$.{}` should be a {expected}, found {}",
            cell_path.join("."),
            value.get_type()
        ),
        value.span(),
    )
}

fn u8_out_of_range(value: i64, cell_path: &[&str], span: Span) -> LabeledError {
    LabeledError::new("invalid config").with_label(
        format!(
            "`$.{}` should be an integer between 0 and 255, found {}",
            cell_path.join("."),
            value
        ),
        span,
    )
}

pub fn positive_integer(value: i64, cell_path: &[&str], span: Span) -> LabeledError {
    LabeledError::new("invalid config").with_label(
        format!(
            "`$.{}` should be a positive integer, found {}",
            cell_path.join("."),
            value
        ),
        span,
    )
}

/// try to parse a bool in the *value* at the given *cell path*
pub fn try_bool(value: &Value, cell_path: &[&str]) -> Result<Option<bool>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Bool { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "bool")),
        _ => Ok(None),
    }
}

/// try to parse a string in the *value* at the given *cell path*
pub fn try_string(value: &Value, cell_path: &[&str]) -> Result<Option<String>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

/// try to parse an integer in the *value* at the given *cell path*
pub fn try_int(value: &Value, cell_path: &[&str]) -> Result<Option<i64>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Int { val, .. }) => Ok(Some(val)),
        Some(x) => Err(invalid_type(&x, cell_path, "int")),
        _ => Ok(None),
    }
}

/// try to parse an ANSI modifier in the *value* at the given *cell path*
pub fn try_modifier(value: &Value, cell_path: &[&str]) -> Result<Option<Modifier>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::Nothing { .. }) => Ok(Some(Modifier::empty())),
        Some(Value::String { val, .. }) => match val.as_str() {
            "" => Ok(Some(Modifier::empty())),
            "bold" => Ok(Some(Modifier::BOLD)),
            "italic" => Ok(Some(Modifier::ITALIC)),
            "underline" => Ok(Some(Modifier::UNDERLINED)),
            "blink" => Ok(Some(Modifier::SLOW_BLINK)),
            x => Err(LabeledError::new(
                "invalid config").with_label(
                format!(
                    r#"`$.{}` should be the empty string, one of [italic, bold, underline, blink] or null, found {}"#,
                    cell_path.join("."),
                    x
                ),
                value.span()
            )),
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string or null")),
        _ => Ok(None),
    }
}

/// try to parse a color in the *value* at the given *cell path*
pub fn try_color(value: &Value, cell_path: &[&str]) -> Result<Option<Color>, LabeledError> {
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
            x => Err(LabeledError::new(
                "invalid config").with_label(
                format!(
                    r#"`$.{}` should be a u8, a list of three u8s or one of [black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white] , found {}"#,
                    cell_path.join("."),
                    x
                ),
                value.span()
            )),
        },
        Some(Value::Int { val, .. }) => {
            if !(0..=255).contains(&val) {
                // FIXME: use a real span?
                return Err(u8_out_of_range(val, cell_path, Span::unknown()));
            }

            Ok(Some(Color::Rgb(val as u8, val as u8, val as u8)))
        }
        Some(Value::List { vals, .. }) => {
            if vals.len() != 3 {
                return Err(LabeledError::new(
                    "invalid config").with_label(
                    format!("`$.{}` is not a valid config field, expected a list of three u8, found {} items", cell_path.join("."), vals.len()),
                    // FIXME: use a real span?
                    Span::unknown(),
                ));
            }

            let mut channels: Vec<u8> = vec![];

            for (i, val) in vals.iter().enumerate() {
                let mut cell_path = cell_path.to_vec().clone();

                let tail = format!("{}", i);
                cell_path.push(&tail);

                match val {
                    Value::Int { val: x, .. } => {
                        if (*x < 0) | (*x > 255) {
                            return Err(u8_out_of_range(*x, &cell_path, val.span()));
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
pub fn try_fg_bg_colors(
    value: &Value,
    cell_path: &[&str],
    default: &BgFgColorConfig,
) -> Result<Option<BgFgColorConfig>, LabeledError> {
    let cell = follow_cell_path(value, cell_path).unwrap();
    let columns = match &cell {
        Value::Record { val: rec, .. } => rec.columns().collect::<Vec<_>>(),
        x => return Err(invalid_type(x, cell_path, "record")),
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
                return Err(invalid_field(&cell_path, cell.span()));
            }
        }
    }

    Ok(Some(colors))
}

/// try to parse a key in the *value* at the given *cell path*
pub fn try_key(value: &Value, cell_path: &[&str]) -> Result<Option<KeyEvent>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => match val.as_str() {
            "up" => Ok(Some(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))),
            "down" => Ok(Some(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))),
            "left" => Ok(Some(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE))),
            "right" => Ok(Some(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))),
            "escape" => Ok(Some(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))),
            x => {
                if x.len() != 1 {
                    if x.len() == 5
                        && (x.starts_with("<c-") || x.starts_with("<C-"))
                        && x.ends_with('>')
                    {
                        #[allow(clippy::iter_nth_zero)]
                        return Ok(Some(KeyEvent::new(
                            // NOTE: this `unwrap` cannot fail because the length of `x` is `5`
                            KeyCode::Char(x.to_string().chars().nth(3).unwrap()),
                            KeyModifiers::CONTROL,
                        )));
                    }

                    return Err(LabeledError::new(
                        "invalid config")
                        .with_label(format!(
                            r#"`$.{}` should be a character, possibly inside '<c-...>' or '<C-...>', or one of [up, down, left, right, escape] , found {}"#,
                            cell_path.join("."),
                            x
                        ),
                        value.span()
                    ));
                }

                #[allow(clippy::iter_nth_zero)]
                Ok(Some(KeyEvent::new(
                    // NOTE: this `unwrap` cannot fail because the length of `x` is `1`
                    KeyCode::Char(x.to_string().chars().nth(0).unwrap()),
                    KeyModifiers::NONE,
                )))
            }
        },
        Some(x) => Err(invalid_type(&x, cell_path, "string")),
        _ => Ok(None),
    }
}

/// try to parse a layout in the *value* at the given *cell path*
pub fn try_layout(value: &Value, cell_path: &[&str]) -> Result<Option<Layout>, LabeledError> {
    match follow_cell_path(value, cell_path) {
        Some(Value::String { val, .. }) => match val.as_str() {
            "table" => Ok(Some(Layout::Table)),
            "compact" => Ok(Some(Layout::Compact)),
            x => Err(LabeledError::new("invalid config").with_label(
                format!(
                    r#"`$.{}` should be one of [table, compact] , found {}"#,
                    cell_path.join("."),
                    x
                ),
                value.span(),
            )),
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
pub fn follow_cell_path(value: &Value, cell_path: &[&str]) -> Option<Value> {
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use nu_protocol::LabeledError;
    use nu_protocol::{record, Record, Value};
    use ratatui::style::{Color, Modifier};

    use super::{
        follow_cell_path, try_bool, try_color, try_fg_bg_colors, try_int, try_key, try_layout,
        try_modifier, try_string,
    };
    use crate::config::{BgFgColorConfig, Layout};

    #[test]
    fn follow_str_cell_path() {
        let inner_record_a = Value::test_int(1);
        let inner_record_b = Value::test_int(2);
        let record = Value::test_record(record! {
            "a" => inner_record_a.clone(),
            "b" => inner_record_b.clone(),
        });
        let string = Value::test_string("some string");
        let int = Value::test_int(123);

        let value = Value::test_record(record! {
            "r" => record.clone(),
            "s" => string.clone(),
            "i" => int.clone(),
        });

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
        assert_eq!(err.labels.len(), 1);
        assert_eq!(err.msg, "invalid config");
        assert_eq!(
            err.labels[0].text,
            format!("`$.{}` {}", cell_path, expected)
        );
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
    fn trying_int() {
        test_tried_error(
            try_int(&Value::test_bool(true), &[]),
            "",
            "should be a int, found bool",
        );
        test_tried_error(
            try_int(&Value::test_string("my string"), &[]),
            "",
            "should be a int, found string",
        );

        assert_eq!(try_int(&Value::test_int(123), &[]), Ok(Some(123)));
        assert_eq!(try_int(&Value::test_int(-123), &[]), Ok(Some(-123)));
        assert_eq!(try_int(&Value::test_int(123), &["x"]), Ok(None));
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
            "should be a character, possibly inside '<c-...>' or '<C-...>', or one of [up, down, left, right, escape] , found enter",
        );

        let cases = vec![
            ("up", KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            ("down", KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            ("left", KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            ("right", KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            ("escape", KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            ("a", KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
            ("b", KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
            ("x", KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            ("x", KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            (
                "<C-x>",
                KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            ),
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
            "should be a string, u8 or [u8, u8, u8], found bool",
        );
        test_tried_error(
            try_color(&Value::test_int(-1), &[]),
            "",
            "should be an integer between 0 and 255, found -1",
        );
        test_tried_error(
            try_color(&Value::test_int(256), &[]),
            "",
            "should be an integer between 0 and 255, found 256",
        );
        test_tried_error(
            try_color(&Value::test_list(vec![]), &[]),
            "",
            "is not a valid config field, expected a list of three u8, found 0 items",
        );
        test_tried_error(
            try_color(&Value::test_list(vec![Value::test_int(1)]), &[]),
            "",
            "is not a valid config field, expected a list of three u8, found 1 items",
        );
        test_tried_error(
            try_color(
                &Value::test_list(vec![Value::test_int(1), Value::test_int(2)]),
                &[],
            ),
            "",
            "is not a valid config field, expected a list of three u8, found 2 items",
        );
        test_tried_error(
            try_color(&Value::test_string("x"), &[]),
            "",
            "should be a u8, a list of three u8s or one of [black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white] , found x",
        );

        let cases = vec![
            (Value::test_string("black"), Color::Black),
            (Value::test_string("red"), Color::Red),
            (Value::test_string("green"), Color::Green),
            (Value::test_string("blue"), Color::Blue),
            (Value::test_int(123), Color::Rgb(123, 123, 123)),
            (
                Value::test_list(vec![
                    Value::test_int(1),
                    Value::test_int(2),
                    Value::test_int(3),
                ]),
                Color::Rgb(1, 2, 3),
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(try_color(&input, &[]), Ok(Some(expected)));
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
                &Value::test_record(record! {
                    "x" => Value::test_nothing(),
                }),
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
            let mut rec = Record::new();
            cols.iter().zip(vals).for_each(|(col, val)| {
                rec.push(*col, val);
            });
            assert_eq!(
                try_fg_bg_colors(&Value::test_record(rec), &[], &default_color),
                Ok(Some(expected))
            );
        }
    }
}
