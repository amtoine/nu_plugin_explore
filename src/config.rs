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
