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

use std::cell::LazyCell;

pub use button::*;
pub use either_of;
#[cfg(feature = "image")]
pub use image::*;
#[cfg(feature = "input")]
pub use input::*;
pub use list_view::*;
pub use notification::*;
use rooibos_component_macros::ReactiveTheme;
use rooibos_dom::BorderType;
use rooibos_reactive::IntoSignal;
use rooibos_reactive::graph::graph::ReactiveNode;
use rooibos_reactive::graph::signal::ArcTrigger;
use rooibos_reactive::graph::traits::{Get, Track};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_theme::{Color, SetTheme, Theme};
pub use show::*;
pub use tab_view::*;
#[cfg(all(feature = "terminal-widget", not(target_arch = "wasm32")))]
pub use terminal::*;
pub use wrapping_list::*;

#[derive(ReactiveTheme, Theme, Clone, Copy, Default, Debug)]
#[theme(prefix = "__internal")]
pub struct ColorTheme {
    pub text_primary: Color,
    pub active: Color,
    pub disabled_light: Color,
    pub disabled_dark: Color,
    pub border: Color,
    pub border_focused: Color,
    pub border_disabled: Color,
}

#[derive(ReactiveTheme, Theme, Clone, Copy, Default, Debug)]
#[theme(prefix = "__internal")]
pub struct BorderProperties {
    pub primary: BorderType,
    pub active: BorderType,
    pub hovered: BorderType,
    pub disabled: BorderType,
}

#[derive(ReactiveTheme, Theme, Clone, Copy, Default, Debug)]
#[theme(prefix = "__internal")]
pub struct AppTheme {
    #[subtheme]
    pub color_theme: ColorTheme,
    #[subtheme]
    pub border_properties: BorderProperties,
}

pub struct ThemeSignal {
    trigger: ArcTrigger,
}

thread_local! {
    static THEME: LazyCell<ThemeSignal> = LazyCell::new(|| {
        let theme = AppTheme {
            color_theme: ColorTheme {
                text_primary: Color::Reset,
                active: Color::Green,
                disabled_light: Color::Gray,
                disabled_dark: Color::DarkGray,
                border: Color::Gray,
                border_focused: Color::Blue,
                border_disabled: Color::DarkGray,
            },
            border_properties: BorderProperties {
                primary: BorderType::Round,
                active: BorderType::Double,
                hovered: BorderType::Double,
                disabled: BorderType::Inner,
            }
        };
        SetTheme::set_global(&theme);
        ThemeSignal { trigger: ArcTrigger::new() }

    });
}

pub fn with_theme<F, T>(f: F) -> Signal<T>
where
    F: Fn(&AppTheme) -> T + Clone + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    THEME.with(|s| s.with_theme(f))
}

impl ThemeSignal {
    pub fn load_theme(&self) -> Signal<ColorTheme> {
        let signal = self.trigger.clone();
        (move || {
            signal.track();
            ColorTheme::current()
        })
        .signal()
    }

    pub fn load_props(&self) -> Signal<BorderProperties> {
        let signal = self.trigger.clone();
        (move || {
            signal.track();
            BorderProperties::current()
        })
        .signal()
    }

    pub fn with_theme<F, T>(&self, f: F) -> Signal<T>
    where
        F: Fn(&AppTheme) -> T + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let signal = self.trigger.clone();
        (move || {
            signal.track();
            AppTheme::with_theme(f.clone())
        })
        .signal()
    }
}
