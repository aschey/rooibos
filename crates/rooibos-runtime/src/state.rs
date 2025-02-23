use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use background_service::{Manager, ServiceContext};
use rooibos_dom::{ViewportSize, on_window_focus_changed, with_nodes_mut};
use rooibos_reactive::graph::signal::{ArcReadSignal, ReadSignal, arc_signal};
use rooibos_reactive::graph::traits::Set;
use tokio::sync::broadcast;
use tokio::task_local;
use tokio_util::sync::CancellationToken;

use crate::signal_handler::signal;
use crate::{RuntimeCommand, TerminalCommand, wasm_compat};

type RestoreFn = dyn Fn() -> io::Result<()> + Send;

type ExitResultFuture = dyn Future<Output = ExitResult> + Send;

#[derive(Debug, Clone)]
pub struct ExitPayload {
    code: signal::Code,
    error: Option<Arc<Box<dyn Error + Send + Sync>>>,
}

impl ExitPayload {
    pub(crate) fn from_result(
        result: Result<signal::Code, Arc<Box<dyn Error + Send + Sync>>>,
    ) -> Self {
        let code = result.clone().unwrap_or(signal::Code::FAILURE);

        Self {
            code,
            error: result.err(),
        }
    }

    pub fn code(&self) -> signal::Code {
        self.code
    }

    pub fn is_success(&self) -> bool {
        self.code == signal::Code::SUCCESS
    }

    pub fn is_termination_signal(&self) -> bool {
        matches!(
            self.code,
            signal::SIGINT | signal::SIGQUIT | signal::SIGTERM
        )
    }

    pub fn error(&self) -> &Option<Arc<Box<dyn Error + Send + Sync>>> {
        &self.error
    }
}

type BeforeExitFn = dyn Fn(ExitPayload) -> Pin<Box<ExitResultFuture>> + Send;

pub(crate) struct RuntimeState {
    pub(crate) term_tx: broadcast::Sender<rooibos_dom::Event>,
    pub(crate) term_command_tx: broadcast::Sender<TerminalCommand>,
    pub(crate) runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    pub(crate) service_manager: Option<Manager>,
    pub(crate) context: ServiceContext,
    pub(crate) restore_terminal: wasm_compat::Mutex<Box<RestoreFn>>,
    pub(crate) before_exit: wasm_compat::Mutex<Box<BeforeExitFn>>,
    pub(crate) window_size: ArcReadSignal<ViewportSize>,
    pub(crate) window_focused: ArcReadSignal<bool>,
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

        let (window_size, set_window_size) = arc_signal(ViewportSize::default());
        with_nodes_mut(|nodes| nodes.on_window_size_change(move |size| set_window_size.set(size)));

        let (window_focused, set_window_focused) = arc_signal(true);
        on_window_focus_changed(move |focused| {
            set_window_focused.set(focused);
        });

        Self {
            term_tx,
            term_command_tx,
            runtime_command_tx,
            restore_terminal: wasm_compat::Mutex::new(Box::new(|| Ok(()))),
            before_exit: wasm_compat::Mutex::new(Box::new(move |_payload| {
                Box::pin(async move { ExitResult::Exit })
            })),
            context: service_manager.get_context(),
            service_manager: Some(service_manager),
            window_size,
            window_focused,
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

pub fn use_window_size() -> ReadSignal<ViewportSize> {
    with_state(|s| ReadSignal::from(s.window_size.clone()))
}

pub fn use_window_focus() -> ReadSignal<bool> {
    with_state(|s| ReadSignal::from(s.window_focused.clone()))
}
