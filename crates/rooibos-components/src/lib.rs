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

use std::cell::{LazyCell, RefCell};
use std::sync::{Arc, LazyLock, RwLock};

pub use button::*;
pub use either_of;
#[cfg(feature = "image")]
pub use image::*;
#[cfg(feature = "input")]
pub use input::*;
pub use list_view::*;
pub use notification::*;
use ratatui::style::Stylize;
use rooibos_dom::BorderType;
use rooibos_reactive::derive_signal;
use rooibos_reactive::graph::signal::ArcRwSignal;
use rooibos_reactive::graph::traits::Track;
use rooibos_reactive::graph::wrappers::read::Signal;
pub use show::*;
pub use tab_view::*;
#[cfg(all(feature = "terminal-widget", not(target_arch = "wasm32")))]
pub use terminal::*;
use tui_theme::{ColorTheme, SetTheme, SubTheme, Theme};
pub use wrapping_list::*;

#[derive(ColorTheme, SubTheme, SetTheme, Clone, Copy, Default, Debug)]
pub struct ColorTheme {
    text_primary: tui_theme::Color,
    active: tui_theme::Color,
    disabled_light: tui_theme::Color,
    disabled_dark: tui_theme::Color,
    border: tui_theme::Color,
    border_focused: tui_theme::Color,
    border_disabled: tui_theme::Color,
}

#[derive(SubTheme, SetTheme, Clone, Copy, Default, Debug)]
pub struct AppProperties {
    border_type_primary: BorderType,
    border_type_active: BorderType,
    border_type_hovered: BorderType,
    border_type_disabled: BorderType,
}

#[derive(Theme, Clone, Copy, Default, Debug)]
pub struct AppTheme {
    color_theme: ColorTheme,
    app_properties: AppProperties,
}

pub struct ThemeSignal {
    signal: ArcRwSignal<()>,
}

thread_local! {
    static THEME: LazyCell<ThemeSignal> = LazyCell::new(|| {
        let theme = AppTheme {
            color_theme: ColorTheme {
                text_primary: tui_theme::Color::Reset,
                active: tui_theme::Color::Green,
                disabled_light: tui_theme::Color::Gray,
                disabled_dark: tui_theme::Color::DarkGray,
                border: tui_theme::Color::Gray,
                border_focused: tui_theme::Color::Blue,
                border_disabled: tui_theme::Color::DarkGray,
            },
            app_properties: AppProperties {
                border_type_primary: BorderType::Round,
                border_type_active: BorderType::Double,
                border_type_hovered: BorderType::Double,
                border_type_disabled: BorderType::Inner,
            }
        };
        theme.set_global();
        ThemeSignal { signal: ArcRwSignal::new(()) }

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
        let signal = self.signal.clone();
        derive_signal!({
            signal.track();
            ColorTheme::current()
        })
    }

    pub fn load_props(&self) -> Signal<AppProperties> {
        let signal = self.signal.clone();
        derive_signal!({
            signal.track();
            AppProperties::current()
        })
    }

    pub fn with_theme<F, T>(&self, f: F) -> Signal<T>
    where
        F: Fn(&AppTheme) -> T + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let signal = self.signal.clone();
        derive_signal!({
            signal.track();
            AppTheme::with_theme(f.clone())
        })
    }
}
