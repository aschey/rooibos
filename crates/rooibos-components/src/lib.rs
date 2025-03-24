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
    primary: tui_theme::Color,
    active_highlight: tui_theme::Color,
    disabled_fg: tui_theme::Color,
    disabled_bg: tui_theme::Color,
    border: tui_theme::Color,
    focused_border: tui_theme::Color,
    disabled_border: tui_theme::Color,
}

#[derive(SubTheme, SetTheme, Clone, Copy, Default, Debug)]
pub struct AppProperties {
    button_borders: BorderType,
    active_button_borders: BorderType,
    hovered_button_borders: BorderType,
    disabled_button_borders: BorderType,
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
                primary: tui_theme::Color::Reset,
                active_highlight: tui_theme::Color::Green,
                disabled_fg: tui_theme::Color::Gray,
                disabled_bg: tui_theme::Color::DarkGray,
                border: tui_theme::Color::Gray,
                focused_border: tui_theme::Color::Blue,
                disabled_border: tui_theme::Color::DarkGray,
            },
            app_properties: AppProperties {
                button_borders: BorderType::Round,
                active_button_borders: BorderType::Double,
                hovered_button_borders: BorderType::Double,
                disabled_button_borders: BorderType::Inner,
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
