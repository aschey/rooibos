mod borders;
mod dom;
pub mod events;
mod macros;
#[cfg(not(target_arch = "wasm32"))]
mod nonblocking_terminal;
#[cfg(target_arch = "wasm32")]
mod nonblocking_terminal_wasm;
pub mod widgets;

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

pub fn set_pixel_size(window_size: Option<WindowSize>) -> Result<(), Option<Size>> {
    let pixel_size = window_size.map(|s| ratatui::layout::Size {
        width: s.pixels.width / s.columns_rows.width,
        height: s.pixels.height / s.columns_rows.height,
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
