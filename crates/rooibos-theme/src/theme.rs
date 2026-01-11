use crate::{Color, Theme};

#[derive(Theme, Default, Clone, Debug)]
pub struct AppTheme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub foreground: Color,
    pub background: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

pub const ANSI: AppTheme = AppTheme {
    primary: Color::Cyan,
    secondary: Color::Blue,
    accent: Color::Magenta,
    foreground: Color::Reset,
    background: Color::Reset,
    success: Color::Green,
    warning: Color::Yellow,
    error: Color::Red,
};
