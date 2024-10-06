mod dom;
mod error_boundary;
mod for_loop;
mod macros;
mod provider;
mod suspense;
mod widgets;
pub mod graph {
    pub use reactive_graph::*;
}
use std::cell::{LazyCell, OnceCell};
use std::future::Future;
use std::ops::Deref;
use std::panic::{set_hook, take_hook};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use dom::*;
pub use error_boundary::*;
pub use for_loop::*;
pub use provider::*;
use ratatui::layout::Size;
#[doc(hidden)]
pub use reactive_graph as __reactive;
use reactive_graph::owner::Owner;
pub use suspense::*;
pub use tachys::reactive_graph as __tachys_reactive;
pub use tachys::view::any_view;
pub use throw_error::*;
pub use widgets::*;

pub fn execute_with_owner<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
    set_panic_hook(owner.clone());
    let res = owner.with(f);

    owner.cleanup();
    drop(owner);
    res
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run_with_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    init_executor();
    let local = tokio::task::LocalSet::new();
    local.run_until(f).await
}

#[cfg(target_arch = "wasm32")]
pub async fn run_with_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    init_executor();
    f.await
}

fn set_panic_hook(owner: Owner) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            owner.cleanup();
            original_hook(panic_info);
        }));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_executor() {
    any_spawner::Executor::init_tokio().expect("executor already initialized");
}

#[cfg(target_arch = "wasm32")]
pub fn init_executor() {
    any_spawner::Executor::init_wasm_bindgen().expect("executor already initialized");
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
