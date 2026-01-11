use super::{Modifier, Style};
use crate::Color;

pub trait Styled {
    type Item;

    fn style(&self) -> Style;

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item;
}

macro_rules! color {
    ($variant:expr, $color:ident(), $on_color:ident() -> $ty:ty) => {
        fn $color(self) -> $ty {
            self.fg($variant)
        }

        fn $on_color(self) -> $ty {
            self.bg($variant)
        }
    };

    (pub const $variant:expr, $color:ident(), $on_color:ident() -> $ty:ty) => {
        pub fn $color(self) -> $ty {
            self.fg($variant)
        }

        pub fn $on_color(self) -> $ty {
            self.bg($variant)
        }
    };
}

macro_rules! modifier {
    ($variant:expr, $modifier:ident(), $not_modifier:ident() -> $ty:ty) => {
        fn $modifier(self) -> $ty {
            self.add_modifier($variant)
        }

        fn $not_modifier(self) -> $ty {
            self.remove_modifier($variant)
        }
    };

    (pub const $variant:expr, $modifier:ident(), $not_modifier:ident() -> $ty:ty) => {
        pub const fn $modifier(self) -> $ty {
            self.add_modifier($variant)
        }

        pub const fn $not_modifier(self) -> $ty {
            self.remove_modifier($variant)
        }
    };
}

pub trait Stylize<'a, T>: Sized {
    fn bg<C: Into<Color>>(self, color: C) -> T;
    fn fg<C: Into<Color>>(self, color: C) -> T;
    fn underline_color<C: Into<Color>>(self, color: C) -> T;
    fn reset(self) -> T;
    fn add_modifier(self, modifier: Modifier) -> T;
    fn remove_modifier(self, modifier: Modifier) -> T;

    color!(Color::Black, black(), on_black() -> T);
    color!(Color::Red, red(), on_red() -> T);
    color!(Color::Green, green(), on_green() -> T);
    color!(Color::Yellow, yellow(), on_yellow() -> T);
    color!(Color::Blue, blue(), on_blue() -> T);
    color!(Color::Magenta, magenta(), on_magenta() -> T);
    color!(Color::Cyan, cyan(), on_cyan() -> T);
    color!(Color::Gray, gray(), on_gray() -> T);
    color!(Color::DarkGray, dark_gray(), on_dark_gray() -> T);
    color!(Color::LightRed, light_red(), on_light_red() -> T);
    color!(Color::LightGreen, light_green(), on_light_green() -> T);
    color!(Color::LightYellow, light_yellow(), on_light_yellow() -> T);
    color!(Color::LightBlue, light_blue(), on_light_blue() -> T);
    color!(Color::LightMagenta, light_magenta(), on_light_magenta() -> T);
    color!(Color::LightCyan, light_cyan(), on_light_cyan() -> T);
    color!(Color::White, white(), on_white() -> T);

    modifier!(Modifier::BOLD, bold(), not_bold() -> T);
    modifier!(Modifier::DIM, dim(), not_dim() -> T);
    modifier!(Modifier::ITALIC, italic(), not_italic() -> T);
    modifier!(Modifier::UNDERLINED, underlined(), not_underlined() -> T);
    modifier!(Modifier::SLOW_BLINK, slow_blink(), not_slow_blink() -> T);
    modifier!(Modifier::RAPID_BLINK, rapid_blink(), not_rapid_blink() -> T);
    modifier!(Modifier::REVERSED, reversed(), not_reversed() -> T);
    modifier!(Modifier::HIDDEN, hidden(), not_hidden() -> T);
    modifier!(Modifier::CROSSED_OUT, crossed_out(), not_crossed_out() -> T);
}

impl<T, U> Stylize<'_, T> for U
where
    U: Styled<Item = T>,
{
    fn bg<C: Into<Color>>(self, color: C) -> T {
        let style = self.style().bg(color.into());
        self.set_style(style)
    }

    fn fg<C: Into<Color>>(self, color: C) -> T {
        let style = self.style().fg(color.into());
        self.set_style(style)
    }

    fn underline_color<C: Into<Color>>(self, color: C) -> T {
        let style = self.style().underline_color(color.into());
        self.set_style(style)
    }

    fn add_modifier(self, modifier: Modifier) -> T {
        let style = self.style().add_modifier(modifier);
        self.set_style(style)
    }

    fn remove_modifier(self, modifier: Modifier) -> T {
        let style = self.style().remove_modifier(modifier);
        self.set_style(style)
    }

    fn reset(self) -> T {
        self.set_style(Style::reset())
    }
}

impl Styled for Style {
    type Item = Self;
    fn style(&self) -> Style {
        *self
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        style.into()
    }
}

impl<T> Styled for T
where
    T: ratatui::style::Styled,
{
    type Item = <T as ratatui::style::Styled>::Item;

    fn style(&self) -> Style {
        self.style().into()
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.set_style(style.into())
    }
}
