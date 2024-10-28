mod dom;
pub mod events;
mod macros;
pub mod widgets;

use std::cell::{LazyCell, OnceCell};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub use dom::*;
use ratatui::layout::Size;
#[doc(hidden)]
pub use ratatui::text as __text;
pub use terminput::*;

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
    static EDITING: LazyCell<Arc<AtomicBool>> = const { LazyCell::new(move || Arc::new(AtomicBool::new(false))) };
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

pub fn set_editing(editing: bool) {
    EDITING.with(|e| e.store(editing, Ordering::Relaxed));
}

pub fn is_editing() -> bool {
    EDITING.with(|e| e.load(Ordering::Relaxed))
}

pub fn editing() -> Arc<AtomicBool> {
    EDITING.with(|e| e.deref().clone())
}
