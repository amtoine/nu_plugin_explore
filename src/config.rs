use console::Key;
use ratatui::style::{Color, Modifier};

pub(super) struct ColorConfig {
    pub normal: BgFgColorConfig,
    pub selected: BgFgColorConfig,
    pub selected_modifier: Modifier,
    pub selected_symbol: String,
    pub status_bar: BgFgColorConfig,
}

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
    pub peeking: PeekingBindingsMap
}

pub(super) struct Config {
    pub colors: ColorConfig,
    pub keybindings: KeyBindingsMap,
    pub show_cell_path: bool,
}

impl Config {
    pub(super) fn default() -> Config {
        Config {
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
