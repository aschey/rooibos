mod color;
mod local_override;
pub mod palette;
mod style;
mod theme;

// hack for referencing the current crate in proc macros
// https://github.com/bkchr/proc-macro-crate/issues/14#issuecomment-1742071768
extern crate self as rooibos_theme;

use std::ops::{Deref, DerefMut, Index};

pub use color::*;
use rooibos_reactive::graph::signal::ArcTrigger;
pub use rooibos_theme_macros::*;
pub use style::*;
pub use theme::*;
pub mod profile {
    pub use termprofile::{DetectorSettings, IsTerminal, QueryTerminal, TermVars};
}

#[derive(Clone, Default)]
pub struct ThemeContext {
    pub trigger: ArcTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Theme, Default)]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

impl From<terminal_colorsaurus::ThemeMode> for ColorScheme {
    fn from(value: terminal_colorsaurus::ThemeMode) -> Self {
        match value {
            terminal_colorsaurus::ThemeMode::Dark => ColorScheme::Dark,
            terminal_colorsaurus::ThemeMode::Light => ColorScheme::Light,
        }
    }
}

pub struct Adaptive<T>(pub Light<T>, pub Dark<T>);

impl<T> Adaptive<T> {
    pub fn adapt(&self) -> &T {
        match ColorScheme::current() {
            ColorScheme::Light => &self.0.0,
            ColorScheme::Dark => &self.1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProfileVariant<T> {
    default_value: T,
    ansi_256: Option<T>,
    ansi_16: Option<T>,
    ascii: Option<T>,
    no_tty: Option<T>,
}

impl<T> ProfileVariant<T> {
    pub fn new(default_value: T) -> Self {
        Self {
            default_value,
            ansi_256: None,
            ansi_16: None,
            ascii: None,
            no_tty: None,
        }
    }

    pub fn ansi_256(mut self, value: T) -> Self {
        self.ansi_256 = Some(value);
        self
    }

    pub fn ansi_16(mut self, value: T) -> Self {
        self.ansi_16 = Some(value);
        self
    }

    pub fn ascii(mut self, value: T) -> Self {
        self.ascii = Some(value);
        self
    }

    pub fn no_tty(mut self, value: T) -> Self {
        self.no_tty = Some(value);
        self
    }

    pub fn adapt(self) -> T {
        let current_profile = TermProfile::current();
        if current_profile <= TermProfile::NoTty
            && let Some(no_tty) = self.no_tty
        {
            return no_tty;
        }
        if current_profile <= TermProfile::NoColor
            && let Some(ascii) = self.ascii
        {
            return ascii;
        }
        if current_profile <= TermProfile::Ansi16
            && let Some(ansi_16) = self.ansi_16
        {
            return ansi_16;
        }
        if current_profile <= TermProfile::Ansi256
            && let Some(ansi_256) = self.ansi_256
        {
            return ansi_256;
        }

        self.default_value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeArray<const N: usize>(pub [Color; N]);

#[derive(Debug, Clone, Copy)]
pub struct Dark<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct Light<T>(pub T);

impl<const N: usize> Deref for ThemeArray<N> {
    type Target = [Color; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for ThemeArray<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Index<(Light<usize>, Dark<usize>)> for ThemeArray<N> {
    type Output = Color;

    fn index(&self, (light, dark): (Light<usize>, Dark<usize>)) -> &Self::Output {
        if ColorScheme::current() == ColorScheme::Light {
            &self.0[light.0]
        } else {
            &self.0[dark.0]
        }
    }
}

impl<const N: usize> Index<(Dark<usize>, Light<usize>)> for ThemeArray<N> {
    type Output = Color;

    fn index(&self, (dark, light): (Dark<usize>, Light<usize>)) -> &Self::Output {
        if ColorScheme::current() == ColorScheme::Light {
            &self.0[light.0]
        } else {
            &self.0[dark.0]
        }
    }
}

impl From<ProfileVariant<Color>> for Color {
    fn from(value: ProfileVariant<Color>) -> Self {
        value.adapt()
    }
}

impl From<ProfileVariant<Style>> for Style {
    fn from(value: ProfileVariant<Style>) -> Self {
        value.adapt()
    }
}

impl<T> SetTheme for Adaptive<T>
where
    T: SetTheme,
{
    type Theme = T::Theme;

    fn set(&self) {
        self.adapt().set();
    }

    fn current() -> Self::Theme {
        T::current()
    }

    fn with_theme<F, R>(f: F) -> R
    where
        F: FnOnce(&Self::Theme) -> R,
    {
        T::with_theme(f)
    }
}

pub trait SetTheme {
    type Theme;

    fn set(&self);

    fn current() -> Self::Theme;

    fn with_theme<F, T>(f: F) -> T
    where
        F: FnOnce(&Self::Theme) -> T;
}
