use ratatui::style::{
    palette::{material, tailwind},
    Color, Modifier, Style,
};

pub struct Theme {
    pub foreground: Style,
    pub background: Style,
    pub app_title: Style,
    pub borders: Style,
    pub text: Style,
    pub title: Style,
    pub highlight: Style,
}

pub const THEME: Theme = Theme {
    background: Style::new().bg(BACKGROUND),
    foreground: Style::new().fg(FOREGROUND),
    app_title: Style::new().fg(APP_TITLE).bg(BACKGROUND),
    borders: Style::new().fg(BORDERS),
    text: Style::new().fg(TEXT_FG).bg(TEXT_BG),
    title: Style::new().fg(TITLE),
    highlight: Style::new().bg(BACKGROUND).fg(TEXT_FG),
};

const BACKGROUND: Color = Color::Rgb(28, 32, 25);
const FOREGROUND: Color = Color::Rgb(167, 167, 167);
const C0: Color = Color::Rgb(28, 32, 25);
const BORDERS: Color = Color::Rgb(207, 106, 76);
const C2: Color = Color::Rgb(143, 157, 106);
const TEXT_BG: Color = Color::Rgb(249, 238, 152);
const TITLE: Color = Color::Rgb(117, 135, 166);
const C5: Color = Color::Rgb(155, 133, 157);
const TEXT_FG: Color = Color::Rgb(175, 196, 219);
const C7: Color = Color::Rgb(117, 135, 166);
const C8: Color = Color::Rgb(95, 90, 96);
const APP_TITLE: Color = Color::Rgb(238, 238, 238);
