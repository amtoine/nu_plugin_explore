use ratatui::style::Color;

pub(super) struct StatusBarConfig {
    pub background: Color,
    pub foreground: Color,
}

pub(super) struct NavigationBindingsMap {
    pub up: char,
    pub down: char,
    pub left: char,
    pub right: char,
}

pub(super) struct KeyBindingsMap {
    pub quit: char,
    pub insert: char,
    pub normal: char,
    pub navigation: NavigationBindingsMap,
}

pub(super) struct Config {
    pub status_bar: StatusBarConfig,
    pub keybindings: KeyBindingsMap,
    pub show_cell_path: bool,
}
