use std::future::Future;
use std::panic::{set_hook, take_hook};

pub use background_service::ServiceContext;
pub use commands::*;
use rooibos_dom::Event;
use rooibos_reactive::graph::owner::Owner;
use rooibos_reactive::graph::signal::{signal, ReadSignal};
use rooibos_reactive::graph::traits::{IsDisposed, Set};
pub use runtime::*;
pub use settings::*;
pub use state::*;
use tokio::task;
pub use tokio_util::sync::CancellationToken;
use tracing::error;

pub mod backend;
mod commands;
mod debounce;
pub mod error;
mod input_handler;
mod runtime;
mod settings;
#[cfg(not(target_arch = "wasm32"))]
mod signal_handler;
mod state;

pub mod wasm_compat {
    pub use ::wasm_compat::cell::*;
    pub use ::wasm_compat::futures::*;
    pub use ::wasm_compat::once::*;
    pub use ::wasm_compat::static_init;
    pub use ::wasm_compat::static_init::*;
    pub use ::wasm_compat::sync::*;
    pub use ::wasm_compat::time::*;
}

pub fn execute<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
    set_panic_hook(owner.clone());
    let res = owner.with(f);

    owner.cleanup();
    drop(owner);

    let _ = restore_terminal().inspect_err(|e| error!("error restoring terminal: {e:?}"));
    res
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run_with_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    init_executor();
    let local = task::LocalSet::new();
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

#[cfg(not(target_arch = "wasm32"))]
pub fn init_executor() {
    any_spawner::Executor::init_tokio().expect("executor already initialized");
}

#[cfg(target_arch = "wasm32")]
pub fn init_executor() {
    any_spawner::Executor::init_wasm_bindgen().expect("executor already initialized");
}

pub fn use_keypress() -> ReadSignal<Option<rooibos_dom::KeyEvent>> {
    let mut term_rx = with_state(|s| s.term_tx.subscribe());
    let (term_signal, set_term_signal) = signal(None);
    wasm_compat::spawn_local(async move {
        while let Ok(event) = term_rx.recv().await {
            if term_signal.is_disposed() {
                return;
            }
            if let Event::Key(key_event) = event {
                if key_event.kind == rooibos_dom::KeyEventKind::Press {
                    set_term_signal.set(Some(key_event));
                }
            }
        }
    });
    term_signal
}

pub fn set_panic_hook(owner: Owner) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = restore_terminal();
            owner.cleanup();
            original_hook(panic_info);
        }));
    }

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}
