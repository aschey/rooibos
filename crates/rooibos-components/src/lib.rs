mod button;
#[cfg(feature = "image")]
mod image;
#[cfg(feature = "input")]
mod input;
mod list_view;
mod notification;
mod show;
#[cfg(feature = "spinner")]
pub mod spinner;
mod tab_view;
#[cfg(all(feature = "terminal-widget", not(target_arch = "wasm32")))]
mod terminal;
mod wrapping_list;

pub use button::*;
pub use either_of;
#[cfg(feature = "image")]
pub use image::*;
#[cfg(feature = "input")]
pub use input::*;
pub use list_view::*;
pub use notification::*;
use rooibos_dom::BorderType;
use rooibos_theme::{Color, Theme};
pub use show::*;
pub use tab_view::*;
#[cfg(all(feature = "terminal-widget", not(target_arch = "wasm32")))]
pub use terminal::*;
pub use wrapping_list::*;

#[derive(Theme, Clone, Copy, Default, Debug)]
pub struct ColorTheme {
    pub text_primary: Color,
    pub active: Color,
    pub disabled_light: Color,
    pub disabled_dark: Color,
    pub border: Color,
    pub border_focused: Color,
    pub border_disabled: Color,
}

#[derive(Theme, Clone, Copy, Default, Debug)]
pub struct BorderProperties {
    pub primary: BorderType,
    pub active: BorderType,
    pub hovered: BorderType,
    pub disabled: BorderType,
}

#[derive(Theme, Clone, Copy, Default, Debug)]
pub struct AppTheme {
    #[subtheme]
    pub color_theme: ColorTheme,
    #[subtheme]
    pub border_properties: BorderProperties,
}

pub fn default_theme() -> AppTheme {
    AppTheme {
        color_theme: ColorTheme {
            text_primary: Color::Reset,
            active: Color::Green,
            disabled_light: Color::Gray,
            disabled_dark: Color::DarkGray,
            border: rooibos_theme::Color::Gray,
            border_focused: Color::Blue,
            border_disabled: Color::DarkGray,
        },
        border_properties: BorderProperties {
            primary: BorderType::Round,
            active: BorderType::Double,
            hovered: BorderType::Double,
            disabled: BorderType::Inner,
        },
    }
}
