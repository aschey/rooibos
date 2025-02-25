pub mod dom;
mod error_boundary;
mod for_loop;
mod macros;
mod provider;
mod suspense;
mod widgets;
pub mod graph {
    pub use reactive_graph::*;
}
pub mod stores {
    pub use reactive_stores::*;
}
use std::future::Future;
use std::panic::{set_hook, take_hook};

use any_spawner::Executor;
pub use error_boundary::*;
pub use for_loop::*;
pub use provider::*;
#[doc(hidden)]
pub use reactive_graph as __reactive;
use reactive_graph::owner::Owner;
#[cfg(feature = "effects")]
pub use rooibos_dom::tachyonfx;
pub use rooibos_dom::{IntoSpan, NonblockingTerminal};
pub use suspense::*;
pub use tachys::reactive_graph as __tachys_reactive;
pub use tachys::view::any_view;
pub use terminput::*;
pub mod error {
    pub use throw_error::*;
    pub type Result<T> = core::result::Result<T, throw_error::Error>;
}

#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub use tokio as __tokio;
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen as __wasm_bindgen;
pub use widgets::*;

pub fn execute_with_owner<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
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

pub fn install_panic_hook() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let owner = Owner::current().unwrap();
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

pub async fn tick() {
    Executor::tick().await;
}
