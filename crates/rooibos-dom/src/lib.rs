mod dom;
mod error_boundary;
mod events;
mod suspense;
mod widgets;

use std::cell::OnceCell;
use std::future::Future;
use std::time::Duration;

pub use dom::*;
pub use error_boundary::*;
pub use events::*;
use ratatui::layout::Size;
#[doc(hidden)]
pub use ratatui::text as __text;
#[doc(hidden)]
pub use ratatui::widgets as __widgets;
#[doc(hidden)]
pub use reactive_graph as __reactive;
pub use suspense::*;
pub use tachys::reactive_graph as __tachys_reactive;
pub use terminput::*;
pub use throw_error::*;
pub use widgets::*;

#[macro_export]
macro_rules! derive_signal {
    ($($x:tt)*) => {
        $crate::__reactive::wrappers::read::Signal::derive(move || $($x)*)
    };
}

#[macro_export]
macro_rules! line {
    () => {
        $crate::__text::Line::default()
    };
    ($string:literal) => {
        $crate::__text::Line::from(format!($string))
    };
    ($span:expr) => {
        $crate::__text::Line::from($span)
    };
    ($span:expr; $n:expr) => {
        $crate::__text::Line::from(vec![$span.into(); $n])
    };
    ($($span:expr),+ $(,)?) => {{
        $crate::__text::Line::from(vec![
        $(
            $span.into(),
        )+
        ])
    }};
}

#[macro_export]
macro_rules! text {
    () => {
        $crate::__text::Text::default()
    };
    ($string:literal) => {
        $crate::__text::Text::from(format!($string))
    };
    ($line:expr; $n:expr) => {
        $crate::__text::::Text::from(vec![$line.into(); $n])
    };
    ($($line:expr),+ $(,)?) => {{
        $crate::__text::Text::from(vec![
        $(
            $line.into(),
        )+
        ])
    }};
}

#[macro_export]
macro_rules! span {
    ($string:literal) => {
        $crate::__text::Span::raw(format!($string))
    };
    ($expr:expr) => {
        $crate::__text::Span::raw($expr.to_string())
    };
    ($string:literal, $($arg:tt)*) => {
        $crate::__text::Span::raw(format!($string, $($arg)*))
    };
    ($style:expr, $($arg:tt)*) => {
        compile_error!("first parameter must be a formatting specifier followed by a comma OR a `Style` followed by a semicolon")
    };
    ($style:expr; $($arg:tt)*) => {
        $crate::__text::Span::styled(format!($($arg)*), $style)
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Focused,
    Active,
    Default,
}

pub fn delay<F>(duration: Duration, f: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_compat::futures::spawn_local(async move {
        wasm_compat::futures::sleep(duration).await;
        f.await;
    });
}

thread_local! {
    static SUPPORTS_KEYBOARD_ENHANCEMENT: OnceCell<bool> = const { OnceCell::new() };
    static PIXEL_SIZE: OnceCell<Option<Size>> = const { OnceCell::new() };
}

pub fn set_supports_keyboard_enhancement(supports_keyboard_enhancement: bool) -> Result<(), bool> {
    SUPPORTS_KEYBOARD_ENHANCEMENT.with(|s| s.set(supports_keyboard_enhancement))
}

pub fn supports_keyboard_enhancement() -> bool {
    SUPPORTS_KEYBOARD_ENHANCEMENT.with(|s| *s.get().unwrap())
}

pub fn set_pixel_size(pixel_size: Option<Size>) -> Result<(), Option<Size>> {
    PIXEL_SIZE.with(|p| p.set(pixel_size))
}

pub fn pixel_size() -> Option<Size> {
    PIXEL_SIZE.with(|p| *p.get().unwrap())
}
