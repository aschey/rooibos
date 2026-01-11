use palette::bool_mask::LazySelect;
use palette::color_difference::Wcag21RelativeContrast;
use palette::num::{Arithmetics, MulSub, PartialCmp, Powf, Real};
use palette::stimulus::IntoStimulus;
use palette::{
    FromColor, Hsl, Hsluv, Hsv, Hwb, Lab, Lch, Lchuv, LinSrgb, Luv, Okhsl, Okhsv, Okhwb, Oklab,
    Oklch, Srgb, Xyz, Yxy,
};

use super::{Color, indexed_to_rgb};
use crate::SetTheme;
use crate::color::{ColorPaletteColorThemeExt, TermProfile};

impl Color {
    pub fn to_rgb_fg(self) -> Srgb<u8> {
        self.to_rgb(true)
    }

    pub fn to_rgb_bg(self) -> Srgb<u8> {
        self.to_rgb(false)
    }

    pub fn to_hex_fg(self) -> String {
        self.to_hex(true)
    }

    pub fn to_hex_bg(self) -> String {
        self.to_hex(false)
    }

    pub fn luminance_fg(&self) -> f32 {
        self.to_rgb_fg()
            .into_linear::<f32>()
            .relative_luminance()
            .luma
    }

    pub fn luminance_bg(&self) -> f32 {
        self.to_rgb_bg()
            .into_linear::<f32>()
            .relative_luminance()
            .luma
    }

    pub fn to_hex(self, is_fg: bool) -> String {
        let rgb = self.to_rgb(is_fg);
        format!("#{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue).to_uppercase()
    }

    fn to_rgb(self, is_fg: bool) -> Srgb<u8> {
        match self {
            Self::Rgb(r, g, b) => Srgb::new(r, g, b),
            Self::Reset => {
                if is_fg {
                    Self::terminal_fg().to_rgb_fg()
                } else {
                    Self::terminal_bg().to_rgb_bg()
                }
            }
            Self::Black => indexed_to_rgb(0),
            Self::Red => indexed_to_rgb(1),
            Self::Green => indexed_to_rgb(2),
            Self::Yellow => indexed_to_rgb(3),
            Self::Blue => indexed_to_rgb(4),
            Self::Magenta => indexed_to_rgb(5),
            Self::Cyan => indexed_to_rgb(6),
            Self::Gray => indexed_to_rgb(7),
            Self::DarkGray => indexed_to_rgb(8),
            Self::LightRed => indexed_to_rgb(9),
            Self::LightGreen => indexed_to_rgb(10),
            Self::LightYellow => indexed_to_rgb(11),
            Self::LightBlue => indexed_to_rgb(12),
            Self::LightMagenta => indexed_to_rgb(13),
            Self::LightCyan => indexed_to_rgb(14),
            Self::White => indexed_to_rgb(15),
            Self::Indexed(idx) => indexed_to_rgb(idx),
        }
    }

    pub fn into_adaptive(self) -> Self {
        if self.is_compatible() {
            return self;
        }
        self.into_tui().map(Into::into).unwrap_or(Self::Reset)
    }

    fn into_anstyle(self) -> Option<anstyle::Color> {
        let value = match self {
            Color::Reset => return None,
            Color::Black => anstyle::Color::Ansi(anstyle::AnsiColor::Black),
            Color::Red => anstyle::Color::Ansi(anstyle::AnsiColor::Red),
            Color::Green => anstyle::Color::Ansi(anstyle::AnsiColor::Green),
            Color::Yellow => anstyle::Color::Ansi(anstyle::AnsiColor::Yellow),
            Color::Blue => anstyle::Color::Ansi(anstyle::AnsiColor::Blue),
            Color::Magenta => anstyle::Color::Ansi(anstyle::AnsiColor::Magenta),
            Color::Cyan => anstyle::Color::Ansi(anstyle::AnsiColor::Cyan),
            Color::Gray => anstyle::Color::Ansi(anstyle::AnsiColor::White),
            Color::DarkGray => anstyle::Color::Ansi(anstyle::AnsiColor::BrightBlack),
            Color::LightRed => anstyle::Color::Ansi(anstyle::AnsiColor::BrightRed),
            Color::LightGreen => anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen),
            Color::LightYellow => anstyle::Color::Ansi(anstyle::AnsiColor::BrightYellow),
            Color::LightBlue => anstyle::Color::Ansi(anstyle::AnsiColor::BrightBlue),
            Color::LightMagenta => anstyle::Color::Ansi(anstyle::AnsiColor::BrightMagenta),
            Color::LightCyan => anstyle::Color::Ansi(anstyle::AnsiColor::BrightCyan),
            Color::White => anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite),
            Color::Indexed(index) => anstyle::Color::Ansi256(anstyle::Ansi256Color(index)),
            Color::Rgb(r, g, b) => anstyle::Color::Rgb(anstyle::RgbColor(r, g, b)),
        };
        let profile = TermProfile::current();
        profile.adapt_color(value)
    }

    fn into_tui(self) -> Option<ratatui::style::Color> {
        let value = match self {
            Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(r, g, b),
            Color::Reset => ratatui::style::Color::Reset,
            Color::Black => ratatui::style::Color::Black,
            Color::Red => ratatui::style::Color::Red,
            Color::Green => ratatui::style::Color::Green,
            Color::Yellow => ratatui::style::Color::Yellow,
            Color::Blue => ratatui::style::Color::Blue,
            Color::Magenta => ratatui::style::Color::Magenta,
            Color::Cyan => ratatui::style::Color::Cyan,
            Color::Gray => ratatui::style::Color::Gray,
            Color::DarkGray => ratatui::style::Color::DarkGray,
            Color::LightRed => ratatui::style::Color::LightRed,
            Color::LightGreen => ratatui::style::Color::LightGreen,
            Color::LightYellow => ratatui::style::Color::LightYellow,
            Color::LightBlue => ratatui::style::Color::LightBlue,
            Color::LightMagenta => ratatui::style::Color::LightMagenta,
            Color::LightCyan => ratatui::style::Color::LightCyan,
            Color::White => ratatui::style::Color::White,
            Color::Indexed(idx) => ratatui::style::Color::Indexed(idx),
        };
        let profile = TermProfile::current();
        profile.adapt_color(value)
    }
}

impl From<Color> for ratatui::style::Color {
    fn from(value: Color) -> Self {
        value.into_tui().unwrap_or(ratatui::style::Color::Reset)
    }
}

impl From<ratatui::style::Color> for Color {
    fn from(value: ratatui::style::Color) -> Self {
        match value {
            ratatui::style::Color::Reset => Color::Reset,
            ratatui::style::Color::Black => Color::Black,
            ratatui::style::Color::Red => Color::Red,
            ratatui::style::Color::Green => Color::Green,
            ratatui::style::Color::Yellow => Color::Yellow,
            ratatui::style::Color::Blue => Color::Blue,
            ratatui::style::Color::Magenta => Color::Magenta,
            ratatui::style::Color::Cyan => Color::Cyan,
            ratatui::style::Color::Gray => Color::Gray,
            ratatui::style::Color::DarkGray => Color::DarkGray,
            ratatui::style::Color::LightRed => Color::LightRed,
            ratatui::style::Color::LightGreen => Color::LightGreen,
            ratatui::style::Color::LightYellow => Color::LightYellow,
            ratatui::style::Color::LightBlue => Color::LightBlue,
            ratatui::style::Color::LightMagenta => Color::LightMagenta,
            ratatui::style::Color::LightCyan => Color::LightCyan,
            ratatui::style::Color::White => Color::White,
            ratatui::style::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
            ratatui::style::Color::Indexed(idx) => Color::Indexed(idx),
        }
    }
}

impl From<anstyle::Color> for Color {
    fn from(value: anstyle::Color) -> Self {
        match value {
            anstyle::Color::Ansi(anstyle::AnsiColor::Black) => Color::Black,
            anstyle::Color::Ansi(anstyle::AnsiColor::Red) => Color::Red,
            anstyle::Color::Ansi(anstyle::AnsiColor::Green) => Color::Green,
            anstyle::Color::Ansi(anstyle::AnsiColor::Yellow) => Color::Yellow,
            anstyle::Color::Ansi(anstyle::AnsiColor::Blue) => Color::Blue,
            anstyle::Color::Ansi(anstyle::AnsiColor::Magenta) => Color::Magenta,
            anstyle::Color::Ansi(anstyle::AnsiColor::Cyan) => Color::Cyan,
            anstyle::Color::Ansi(anstyle::AnsiColor::White) => Color::Gray,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightBlack) => Color::DarkGray,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightRed) => Color::LightRed,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen) => Color::LightGreen,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightYellow) => Color::LightYellow,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightBlue) => Color::LightBlue,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightMagenta) => Color::LightMagenta,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightCyan) => Color::LightCyan,
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite) => Color::White,
            anstyle::Color::Ansi256(anstyle::Ansi256Color(index)) => Color::Indexed(index),
            anstyle::Color::Rgb(rgb_color) => {
                Color::Rgb(rgb_color.r(), rgb_color.g(), rgb_color.b())
            }
        }
    }
}

impl<T: IntoStimulus<u8>> From<Srgb<T>> for Color {
    fn from(color: Srgb<T>) -> Self {
        let (red, green, blue) = color.into_format().into_components();
        Self::Rgb(red, green, blue)
    }
}

impl<T: IntoStimulus<u8>> From<LinSrgb<T>> for Color
where
    T: Real + Powf + MulSub + Arithmetics + PartialCmp + Clone,
    T::Mask: LazySelect<T>,
{
    fn from(color: LinSrgb<T>) -> Self {
        let srgb_color = Srgb::<T>::from_linear(color);
        Self::from(srgb_color)
    }
}

impl From<Color> for Option<anstyle::Color> {
    fn from(value: Color) -> Self {
        value.into_anstyle()
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TryFromColorError {
    #[error("unsupported color conversion")]
    Unsupported,
}

impl TryFrom<Color> for anstyle::Color {
    type Error = TryFromColorError;
    fn try_from(value: Color) -> Result<Self, Self::Error> {
        value.into_anstyle().ok_or(TryFromColorError::Unsupported)
    }
}

macro_rules! from_color {
    ($type:ident) => {
        impl From<$type> for Color {
            fn from(value: $type) -> Self {
                let rgb: Srgb = Srgb::from_color(value);
                rgb.into()
            }
        }
    };
}
impl From<u8> for Color {
    fn from(value: u8) -> Self {
        Self::Indexed(value)
    }
}

from_color!(Hsl);
from_color!(Hsluv);
from_color!(Hsv);
from_color!(Hwb);
from_color!(Lab);
from_color!(Lch);
from_color!(Lchuv);
from_color!(Luv);
from_color!(Okhsl);
from_color!(Okhsv);
from_color!(Okhwb);
from_color!(Oklab);
from_color!(Oklch);
from_color!(Xyz);
from_color!(Yxy);
