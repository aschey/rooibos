mod commands;
mod debounce;
pub mod error;
mod input_handler;
mod runtime;
mod settings;
mod signal_handler;
mod state;

use std::panic::{set_hook, take_hook};

pub use background_service::ServiceContext;
pub use commands::*;
pub use proc_exit;
use rooibos_dom::Event;
#[cfg(feature = "reactive")]
use rooibos_reactive::graph::traits::IsDisposed as _;
#[cfg(feature = "reactive")]
use rooibos_reactive::graph::traits::Set as _;
pub use runtime::*;
pub use settings::*;
pub use signal_handler::*;
pub use state::*;
pub use tokio_util::sync::CancellationToken;

pub mod wasm_compat {
    pub use ::wasm_compat::cell::*;
    pub use ::wasm_compat::futures::*;
    pub use ::wasm_compat::once::*;
    pub use ::wasm_compat::static_init;
    pub use ::wasm_compat::static_init::*;
    pub use ::wasm_compat::sync::*;
    pub use ::wasm_compat::time::*;
}

#[cfg(feature = "reactive")]
pub fn use_keypress() -> rooibos_reactive::graph::signal::ReadSignal<Option<rooibos_dom::KeyEvent>>
{
    let mut term_rx = with_state(|s| s.term_tx.subscribe());
    let (term_signal, set_term_signal) = rooibos_reactive::graph::signal::signal(None);
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

pub fn set_panic_hook() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        rooibos_reactive::install_panic_hook();
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = restore_terminal();
            original_hook(panic_info);
        }));
    }

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}
