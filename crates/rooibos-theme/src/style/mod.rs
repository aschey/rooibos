#[macro_use]
mod stylize;

use bitflags::bitflags;
pub use stylize::*;

use crate::color::TermProfile;
use crate::{Color, SetTheme};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    underline_color: Option<Color>,
    add_modifier: Modifier,
    sub_modifier: Modifier,
}

bitflags! {
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
    pub struct Modifier: u16 {
        const BOLD              = 1<<1;
        const DIM               = 1<<2;
        const ITALIC            = 1<<3;
        const UNDERLINED        = 1<<4;
        const SLOW_BLINK        = 1<<5;
        const RAPID_BLINK       = 1<<6;
        const REVERSED          = 1<<7;
        const HIDDEN            = 1<<8;
        const CROSSED_OUT       = 1<<9;
    }
}

impl Style {
    /// Returns a `Style` with default properties.
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
    }

    /// Returns a `Style` resetting all properties.
    pub const fn reset() -> Self {
        Self {
            fg: Some(Color::Reset),
            bg: Some(Color::Reset),
            underline_color: Some(Color::Reset),
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::all(),
        }
    }

    pub fn fg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.fg = Some(color.into());
        self
    }

    pub fn bg<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.bg = Some(color.into());
        self
    }

    pub fn underline_color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.underline_color = Some(color.into());
        self
    }

    pub const fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.sub_modifier = self.sub_modifier.difference(modifier);
        self.add_modifier = self.add_modifier.union(modifier);
        self
    }

    pub const fn remove_modifier(mut self, modifier: Modifier) -> Self {
        self.add_modifier = self.add_modifier.difference(modifier);
        self.sub_modifier = self.sub_modifier.union(modifier);
        self
    }

    pub fn patch<S: Into<Self>>(mut self, other: S) -> Self {
        let other = other.into();
        self.fg = other.fg.or(self.fg);
        self.bg = other.bg.or(self.bg);

        self.underline_color = other.underline_color.or(self.underline_color);
        self.add_modifier.remove(other.sub_modifier);
        self.add_modifier.insert(other.add_modifier);
        self.sub_modifier.remove(other.add_modifier);
        self.sub_modifier.insert(other.sub_modifier);

        self
    }

    color!(pub const Color::Black, black(), on_black() -> Self);
    color!(pub const Color::Red, red(), on_red() -> Self);
    color!(pub const Color::Green, green(), on_green() -> Self);
    color!(pub const Color::Yellow, yellow(), on_yellow() -> Self);
    color!(pub const Color::Blue, blue(), on_blue() -> Self);
    color!(pub const Color::Magenta, magenta(), on_magenta() -> Self);
    color!(pub const Color::Cyan, cyan(), on_cyan() -> Self);
    color!(pub const Color::Gray, gray(), on_gray() -> Self);
    color!(pub const Color::DarkGray, dark_gray(), on_dark_gray() -> Self);
    color!(pub const Color::LightRed, light_red(), on_light_red() -> Self);
    color!(pub const Color::LightGreen, light_green(), on_light_green() -> Self);
    color!(pub const Color::LightYellow, light_yellow(), on_light_yellow() -> Self);
    color!(pub const Color::LightBlue, light_blue(), on_light_blue() -> Self);
    color!(pub const Color::LightMagenta, light_magenta(), on_light_magenta() -> Self);
    color!(pub const Color::LightCyan, light_cyan(), on_light_cyan() -> Self);
    color!(pub const Color::White, white(), on_white() -> Self);

    modifier!(pub const Modifier::BOLD, bold(), not_bold() -> Self);
    modifier!(pub const Modifier::DIM, dim(), not_dim() -> Self);
    modifier!(pub const Modifier::ITALIC, italic(), not_italic() -> Self);
    modifier!(pub const Modifier::UNDERLINED, underlined(), not_underlined() -> Self);
    modifier!(pub const Modifier::SLOW_BLINK, slow_blink(), not_slow_blink() -> Self);
    modifier!(pub const Modifier::RAPID_BLINK, rapid_blink(), not_rapid_blink() -> Self);
    modifier!(pub const Modifier::REVERSED, reversed(), not_reversed() -> Self);
    modifier!(pub const Modifier::HIDDEN, hidden(), not_hidden() -> Self);
    modifier!(pub const Modifier::CROSSED_OUT, crossed_out(), not_crossed_out() -> Self);
}

impl From<Color> for Style {
    fn from(color: Color) -> Self {
        Self::new().fg(color)
    }
}

impl From<(Color, Color)> for Style {
    fn from((fg, bg): (Color, Color)) -> Self {
        Self::new().fg(fg).bg(bg)
    }
}

impl From<Style> for ratatui::style::Style {
    fn from(val: Style) -> Self {
        let mut ratatui_style = ratatui::style::Style::new();
        if let Some(fg) = val.fg {
            ratatui_style = ratatui_style.fg(fg.into());
        }
        if let Some(bg) = val.bg {
            ratatui_style = ratatui_style.bg(bg.into());
        }
        if let Some(underline) = val.underline_color {
            ratatui_style = ratatui_style.underline_color(underline.into());
        }
        let ratatui_style = ratatui_style
            .add_modifier(val.add_modifier.into())
            .remove_modifier(val.sub_modifier.into());
        let profile = TermProfile::current();
        profile.adapt_style(ratatui_style)
    }
}

impl From<ratatui::style::Style> for Style {
    fn from(value: ratatui::style::Style) -> Self {
        Style {
            fg: value.fg.map(Into::into),
            bg: value.bg.map(Into::into),
            underline_color: value.underline_color.map(Into::into),
            add_modifier: value.add_modifier.into(),
            sub_modifier: value.sub_modifier.into(),
        }
    }
}

impl From<Style> for anstyle::Style {
    fn from(val: Style) -> Self {
        let style = anstyle::Style::new()
            .fg_color(val.fg.and_then(Into::into))
            .bg_color(val.bg.and_then(Into::into))
            .underline_color(val.underline_color.and_then(Into::into));

        let profile = TermProfile::current();
        profile.adapt_style(style)
    }
}

impl From<anstyle::Style> for Style {
    fn from(value: anstyle::Style) -> Self {
        Style {
            fg: value.get_fg_color().map(Into::into),
            bg: value.get_fg_color().map(Into::into),
            underline_color: value.get_underline_color().map(Into::into),
            add_modifier: value.get_effects().into(),
            sub_modifier: Modifier::empty(),
        }
    }
}

impl From<Modifier> for ratatui::style::Modifier {
    fn from(value: Modifier) -> Self {
        let mut modifier = ratatui::style::Modifier::empty();

        if value.intersects(Modifier::BOLD) {
            modifier |= ratatui::style::Modifier::BOLD;
        }
        if value.intersects(Modifier::DIM) {
            modifier |= ratatui::style::Modifier::DIM;
        }
        if value.intersects(Modifier::ITALIC) {
            modifier |= ratatui::style::Modifier::ITALIC;
        }
        if value.intersects(Modifier::UNDERLINED) {
            modifier |= ratatui::style::Modifier::UNDERLINED;
        }
        if value.intersects(Modifier::SLOW_BLINK) {
            modifier |= ratatui::style::Modifier::SLOW_BLINK;
        }
        if value.intersects(Modifier::RAPID_BLINK) {
            modifier |= ratatui::style::Modifier::RAPID_BLINK;
        }
        if value.intersects(Modifier::REVERSED) {
            modifier |= ratatui::style::Modifier::REVERSED;
        }
        if value.intersects(Modifier::HIDDEN) {
            modifier |= ratatui::style::Modifier::HIDDEN;
        }
        if value.intersects(Modifier::CROSSED_OUT) {
            modifier |= ratatui::style::Modifier::CROSSED_OUT;
        }

        modifier
    }
}

impl From<ratatui::style::Modifier> for Modifier {
    fn from(value: ratatui::style::Modifier) -> Self {
        let mut modifiers = Modifier::empty();
        if TermProfile::current() == TermProfile::NoTty {
            return modifiers;
        }

        if value.intersects(ratatui::style::Modifier::BOLD) {
            modifiers |= Modifier::BOLD;
        }
        if value.intersects(ratatui::style::Modifier::DIM) {
            modifiers |= Modifier::DIM;
        }
        if value.intersects(ratatui::style::Modifier::ITALIC) {
            modifiers |= Modifier::ITALIC;
        }
        if value.intersects(ratatui::style::Modifier::UNDERLINED) {
            modifiers |= Modifier::UNDERLINED;
        }
        if value.intersects(ratatui::style::Modifier::SLOW_BLINK) {
            modifiers |= Modifier::SLOW_BLINK;
        }
        if value.intersects(ratatui::style::Modifier::RAPID_BLINK) {
            modifiers |= Modifier::RAPID_BLINK;
        }
        if value.intersects(ratatui::style::Modifier::REVERSED) {
            modifiers |= Modifier::REVERSED;
        }
        if value.intersects(ratatui::style::Modifier::HIDDEN) {
            modifiers |= Modifier::HIDDEN;
        }
        if value.intersects(ratatui::style::Modifier::CROSSED_OUT) {
            modifiers |= Modifier::CROSSED_OUT;
        }

        modifiers
    }
}

impl From<Modifier> for anstyle::Effects {
    fn from(value: Modifier) -> Self {
        let mut modifier = anstyle::Effects::new();
        if value.intersects(Modifier::BOLD) {
            modifier |= anstyle::Effects::BOLD;
        }
        if value.intersects(Modifier::DIM) {
            modifier |= anstyle::Effects::DIMMED;
        }
        if value.intersects(Modifier::ITALIC) {
            modifier |= anstyle::Effects::ITALIC;
        }
        if value.intersects(Modifier::UNDERLINED) {
            modifier |= anstyle::Effects::UNDERLINE;
        }
        if value.intersects(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK) {
            modifier |= anstyle::Effects::BLINK;
        }
        if value.intersects(Modifier::REVERSED) {
            modifier |= anstyle::Effects::INVERT;
        }
        if value.intersects(Modifier::HIDDEN) {
            modifier |= anstyle::Effects::HIDDEN;
        }
        if value.intersects(Modifier::CROSSED_OUT) {
            modifier |= anstyle::Effects::STRIKETHROUGH;
        }

        modifier
    }
}

impl From<anstyle::Effects> for Modifier {
    fn from(value: anstyle::Effects) -> Self {
        let mut modifier = Modifier::empty();
        if value.contains(anstyle::Effects::BOLD) {
            modifier |= Modifier::BOLD;
        }
        if value.contains(anstyle::Effects::DIMMED) {
            modifier |= Modifier::DIM;
        }
        if value.contains(anstyle::Effects::ITALIC) {
            modifier |= Modifier::ITALIC;
        }
        if value.contains(anstyle::Effects::UNDERLINE)
            || value.contains(anstyle::Effects::DOUBLE_UNDERLINE)
            || value.contains(anstyle::Effects::DOTTED_UNDERLINE)
            || value.contains(anstyle::Effects::DASHED_UNDERLINE)
        {
            modifier |= Modifier::UNDERLINED;
        }
        if value.contains(anstyle::Effects::BLINK) {
            modifier |= Modifier::SLOW_BLINK;
        }
        if value.contains(anstyle::Effects::INVERT) {
            modifier |= Modifier::REVERSED;
        }
        if value.contains(anstyle::Effects::HIDDEN) {
            modifier |= Modifier::HIDDEN;
        }
        if value.contains(anstyle::Effects::STRIKETHROUGH) {
            modifier |= Modifier::CROSSED_OUT;
        }

        modifier
    }
}
