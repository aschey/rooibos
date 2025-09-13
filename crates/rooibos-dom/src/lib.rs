mod borders;
mod dom;
pub mod events;
mod macros;
#[cfg(not(target_arch = "wasm32"))]
mod nonblocking_terminal;
#[cfg(target_arch = "wasm32")]
mod nonblocking_terminal_wasm;
pub mod widgets;
mod wrap;

use std::borrow::Cow;
use std::cell::{LazyCell, OnceCell};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub use borders::*;
pub use dom::*;
#[cfg(not(target_arch = "wasm32"))]
pub use nonblocking_terminal::*;
#[cfg(target_arch = "wasm32")]
pub use nonblocking_terminal_wasm::*;
use ratatui::backend::WindowSize;
use ratatui::layout::Size;
#[doc(hidden)]
pub use ratatui::text as __text;
use ratatui::text::{Line, Span, Text};
#[cfg(feature = "effects")]
pub use tachyonfx;
pub use terminput::*;

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
    static EDITING: LazyCell<Arc<AtomicBool>> = const { LazyCell::new(move || Arc::new(AtomicBool::new(false))) };
}

pub fn set_supports_keyboard_enhancement(supports_keyboard_enhancement: bool) -> Result<(), bool> {
    SUPPORTS_KEYBOARD_ENHANCEMENT.with(|s| s.set(supports_keyboard_enhancement))
}

pub fn supports_keyboard_enhancement() -> bool {
    SUPPORTS_KEYBOARD_ENHANCEMENT.with(|s| *s.get().unwrap())
}

pub fn supports_key_up() -> bool {
    cfg!(windows) || supports_keyboard_enhancement()
}

pub fn set_pixel_size(window_size: Option<WindowSize>) -> Result<(), Option<Size>> {
    let pixel_size = window_size.and_then(|s| {
        if s.columns_rows.height > 0 && s.columns_rows.width > 0 {
            Some(ratatui::layout::Size {
                width: s.pixels.width / s.columns_rows.width,
                height: s.pixels.height / s.columns_rows.height,
            })
        } else {
            None
        }
    });
    PIXEL_SIZE.with(|p| p.set(pixel_size))
}

pub fn pixel_size() -> Option<Size> {
    PIXEL_SIZE.with(|p| *p.get().unwrap())
}

pub fn set_editing(editing: bool) {
    EDITING.with(|e| e.store(editing, Ordering::Relaxed));
}

pub fn is_editing() -> bool {
    EDITING.with(|e| e.load(Ordering::Relaxed))
}

pub fn editing() -> Arc<AtomicBool> {
    EDITING.with(|e| e.deref().clone())
}

pub trait IntoSpan<'a> {
    fn into_span(self) -> Span<'a>;
}

impl<'a> IntoSpan<'a> for Span<'a> {
    fn into_span(self) -> Span<'a> {
        self
    }
}

impl<'a> IntoSpan<'a> for &'a str {
    fn into_span(self) -> Span<'a> {
        Span::raw(self)
    }
}

impl<'a> IntoSpan<'a> for Cow<'a, str> {
    fn into_span(self) -> Span<'a> {
        Span::raw(self)
    }
}

impl IntoSpan<'static> for String {
    fn into_span(self) -> Span<'static> {
        Span::raw(self)
    }
}

pub trait IntoText<'a> {
    fn into_text(self) -> Text<'a>;
}

impl<'a> IntoText<'a> for Text<'a> {
    fn into_text(self) -> Text<'a> {
        self
    }
}

impl<'a> IntoText<'a> for Span<'a> {
    fn into_text(self) -> Text<'a> {
        self.into()
    }
}

impl<'a> IntoText<'a> for Line<'a> {
    fn into_text(self) -> Text<'a> {
        self.into()
    }
}

impl<'a> IntoText<'a> for &'a str {
    fn into_text(self) -> Text<'a> {
        Text::raw(self)
    }
}

impl<'a> IntoText<'a> for Cow<'a, str> {
    fn into_text(self) -> Text<'a> {
        Text::raw(self)
    }
}

impl IntoText<'static> for String {
    fn into_text(self) -> Text<'static> {
        Text::raw(self)
    }
}

pub trait IntoLine<'a> {
    fn into_line(self) -> Line<'a>;
}

impl<'a> IntoLine<'a> for Span<'a> {
    fn into_line(self) -> Line<'a> {
        self.into()
    }
}

impl<'a> IntoLine<'a> for Line<'a> {
    fn into_line(self) -> Line<'a> {
        self
    }
}

impl<'a> IntoLine<'a> for &'a str {
    fn into_line(self) -> Line<'a> {
        Line::raw(self)
    }
}

impl<'a> IntoLine<'a> for Cow<'a, str> {
    fn into_line(self) -> Line<'a> {
        Line::raw(self)
    }
}

impl IntoLine<'static> for String {
    fn into_line(self) -> Line<'static> {
        Line::raw(self)
    }
}

macro_rules! impl_primitive {
    ($impl_type:ty) => {
        impl IntoSpan<'static> for $impl_type {
            fn into_span(self) -> Span<'static> {
                Span::raw(self.to_string())
            }
        }

        impl IntoLine<'static> for $impl_type {
            fn into_line(self) -> Line<'static> {
                Line::raw(self.to_string())
            }
        }

        impl IntoText<'static> for $impl_type {
            fn into_text(self) -> Text<'static> {
                Text::raw(self.to_string())
            }
        }
    };
}

impl_primitive!(bool);
impl_primitive!(char);
impl_primitive!(f32);
impl_primitive!(f64);
impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i128);
impl_primitive!(isize);
impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);
impl_primitive!(usize);
