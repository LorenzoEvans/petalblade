use ratatui::style::{
    Color, Modifier, Style,
};

pub struct Theme {
    pub foreground: Style,
    pub background: Style,
    pub app_title: Style,
    pub borders: Style,
    pub active_borders: Style,
    pub text: Style,
    pub title: Style,
    pub highlight: Style,
}

pub const THEME: Theme = Theme {
    background: Style::new().bg(BACKGROUND),
    foreground: Style::new().fg(FOREGROUND),
    app_title: Style::new().fg(APP_TITLE).bg(BACKGROUND).add_modifier(Modifier::BOLD),
    borders: Style::new().fg(BORDERS),
    active_borders: Style::new().fg(HIGHLIGHT_FG),
    text: Style::new().fg(TEXT_FG),
    title: Style::new().fg(TITLE).add_modifier(Modifier::BOLD),
    highlight: Style::new().fg(HIGHLIGHT_FG).add_modifier(Modifier::REVERSED),
};

const BACKGROUND: Color = Color::Rgb(20, 20, 20);
const FOREGROUND: Color = Color::Rgb(180, 180, 180);
const BORDERS: Color = Color::Rgb(100, 100, 100);
const TEXT_FG: Color = Color::Rgb(150, 200, 150);
const TITLE: Color = Color::Rgb(200, 150, 100);
const HIGHLIGHT_FG: Color = Color::Rgb(100, 200, 255);
const APP_TITLE: Color = Color::Rgb(255, 100, 100);
