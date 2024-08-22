use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::pin::Pin;

use background_service::{Manager, ServiceContext};
use tokio::sync::broadcast;
use tokio::task_local;
use tokio_util::sync::CancellationToken;

use crate::{wasm_compat, RuntimeCommand, TerminalCommand};

type RestoreFn = dyn Fn() -> io::Result<()> + Send;

type ExitResultFuture = dyn Future<Output = ExitResult> + Send;

pub(crate) struct RuntimeState {
    pub(crate) term_tx: broadcast::Sender<rooibos_dom::Event>,
    pub(crate) term_command_tx: broadcast::Sender<TerminalCommand>,
    pub(crate) runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    pub(crate) service_manager: Option<Manager>,
    pub(crate) context: ServiceContext,
    pub(crate) restore_terminal: wasm_compat::Mutex<Box<RestoreFn>>,
    pub(crate) before_exit: wasm_compat::Mutex<Box<dyn Fn() -> Pin<Box<ExitResultFuture>> + Send>>,
}

impl RuntimeState {
    fn new() -> Self {
        let (term_tx, _) = broadcast::channel(32);
        let (term_command_tx, _) = broadcast::channel(32);
        let (runtime_command_tx, _) = broadcast::channel(32);
        let cancellation_token = CancellationToken::new();
        let service_manager = Manager::new(
            cancellation_token.clone(),
            background_service::Settings::default(),
        );
        Self {
            term_tx,
            term_command_tx,
            runtime_command_tx,
            restore_terminal: wasm_compat::Mutex::new(Box::new(|| Ok(()))),
            before_exit: wasm_compat::Mutex::new(Box::new(move || {
                Box::pin(async move { ExitResult::Exit })
            })),
            context: service_manager.get_context(),
            service_manager: Some(service_manager),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExitResult {
    Exit,
    PreventExit,
}

wasm_compat::static_init! {
    static STATE: wasm_compat::Lazy<wasm_compat::RwLock<HashMap<u32, RuntimeState>>> = wasm_compat::Lazy::new(|| {
        let mut state = HashMap::new();
        state.insert(0, RuntimeState::new());
        wasm_compat::RwLock::new(state)
    });
}

#[cfg(not(target_arch = "wasm32"))]
wasm_compat::static_init! {
    static EXTERNAL_SIGNALS: wasm_compat::Once<broadcast::Sender<async_signal::Signal>> = wasm_compat::Once::new();
}

task_local! {
    static CURRENT_RUNTIME: u32;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_external_signal_source(
    signals: broadcast::Sender<async_signal::Signal>,
) -> Result<(), broadcast::Sender<async_signal::Signal>> {
    EXTERNAL_SIGNALS.with(|s| s.set(signals))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn get_external_signal_stream() -> Option<broadcast::Receiver<async_signal::Signal>> {
    EXTERNAL_SIGNALS.with(|s| s.get().map(|s| s.subscribe()))
}

pub(crate) fn has_external_signal_stream() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return EXTERNAL_SIGNALS.with(|s| s.get().is_some());
    #[cfg(target_arch = "wasm32")]
    return false;
}

fn current_runtime() -> u32 {
    CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0)
}

pub(crate) fn with_state<F: FnOnce(&RuntimeState) -> T, T>(f: F) -> T {
    STATE.with(|s| f(s.read().get(&current_runtime()).expect("runtime missing")))
}

pub(crate) fn with_all_state<F: FnOnce(&HashMap<u32, RuntimeState>) -> T, T>(f: F) -> T {
    STATE.with(|s| f(&s.read()))
}

pub(crate) fn with_state_mut<F: FnOnce(&mut RuntimeState) -> T, T>(f: F) -> T {
    STATE.with(|s| f(s.write().get_mut(&current_runtime()).unwrap()))
}

pub async fn with_runtime<Fut, T>(id: u32, f: Fut) -> T
where
    Fut: Future<Output = T>,
{
    STATE.with(|s| s.write().insert(id, RuntimeState::new()));

    CURRENT_RUNTIME.scope(id, f).await
}
