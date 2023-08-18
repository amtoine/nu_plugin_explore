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
    pub colors: ColorConfig,
    pub keybindings: KeyBindingsMap,
    pub show_cell_path: bool,
}
