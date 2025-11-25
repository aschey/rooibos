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
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use any_spawner::Executor;
pub use error_boundary::*;
pub use for_loop::*;
pub use provider::*;
#[doc(hidden)]
pub use reactive_graph as __reactive;
use reactive_graph::IntoReactiveValue;
use reactive_graph::computed::ScopedFuture;
use reactive_graph::owner::Owner;
use reactive_graph::signal::signal;
use reactive_graph::traits::{Get, Set};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::events::{EventData, EventHandle, NodeState, StateChangeCause, StateChangeEvent};
#[cfg(feature = "effects")]
pub use rooibos_dom::tachyonfx;
pub use rooibos_dom::{IntoLine, IntoSpan, IntoText, NonblockingTerminal};
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
use tui_theme::{ColorPalette, SetTheme, TermProfile};
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

#[derive(Clone)]
pub struct StateProp<T> {
    pub focused: Arc<dyn Fn(T) -> T + Send + Sync>,
    pub direct_focus: bool,
    pub disabled: Arc<dyn Fn(T) -> T + Send + Sync>,
    pub hovered: Arc<dyn Fn(T) -> T + Send + Sync>,
    pub normal: T,
}

impl<T> Default for StateProp<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            focused: Arc::new(|t| t),
            direct_focus: true,
            disabled: Arc::new(|t| t),
            hovered: Arc::new(|t| t),
            normal: T::default(),
        }
    }
}

impl<T> StateProp<T> {
    pub fn new(normal: T) -> Self {
        Self {
            normal,
            direct_focus: true,
            focused: Arc::new(|t| t),
            disabled: Arc::new(|t| t),
            hovered: Arc::new(|t| t),
        }
    }
}

impl<T> StateProp<T>
where
    T: Clone,
{
    pub fn focused(mut self, focused: impl Fn(T) -> T + Send + Sync + 'static) -> Self {
        self.focused = Arc::new(focused);
        self
    }

    pub fn disabled(mut self, disabled: impl Fn(T) -> T + Send + Sync + 'static) -> Self {
        self.disabled = Arc::new(disabled);
        self
    }

    pub fn hovered(mut self, hovered: impl Fn(T) -> T + Send + Sync + 'static) -> Self {
        self.hovered = Arc::new(hovered);
        self
    }

    pub fn normal(mut self, normal: T) -> Self {
        self.normal = normal;
        self
    }

    pub fn direct_focus(mut self, direct: bool) -> Self {
        self.direct_focus = direct;
        self
    }

    fn apply_state(&self, event: StateChangeEvent, is_direct: bool) -> T {
        let mut value = self.normal.clone();
        if event.state.intersects(NodeState::HOVERED) {
            value = (self.hovered)(value);
        }
        let direct_focus_required = self.direct_focus && event.cause == StateChangeCause::Focus;
        let is_required_direct_focus = direct_focus_required && is_direct;
        if event.state.intersects(NodeState::FOCUSED)
            && (is_required_direct_focus || !direct_focus_required)
        {
            value = (self.focused)(value);
        }
        if event.state.intersects(NodeState::DISABLED) {
            value = (self.disabled)(value);
        }
        value
    }
}

pub fn use_state_prop<T, M>(
    state_prop: impl IntoReactiveValue<Signal<StateProp<T>>, M>,
) -> (Signal<T>, impl Fn(StateChangeEvent, EventData, EventHandle))
where
    T: Clone + Send + Sync + 'static,
{
    let (change_event, set_change_event) = signal(StateChangeEvent {
        state: NodeState::empty(),
        cause: StateChangeCause::Enable,
    });
    let (is_direct, set_is_direct) = signal(false);
    let state_prop = state_prop.into_reactive_value();
    let prop = derive_signal!({
        let state_prop = state_prop.get();
        state_prop.apply_state(change_event.get(), is_direct.get())
    });

    (prop, move |event, data, _| {
        set_change_event.set(event);
        set_is_direct.set(data.is_direct);
    })
}

pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    let fut = ScopedFuture::new(fut);
    let profile = TermProfile::current();
    let palette = ColorPalette::current();

    #[cfg(not(target_arch = "wasm32"))]
    Executor::spawn(async move {
        profile.set_local();
        palette.set_local();
        fut.await
    });

    #[cfg(target_family = "wasm")]
    Executor::spawn_local(fut);
}

pub fn spawn_local(fut: impl Future<Output = ()> + 'static) {
    Executor::spawn_local(fut)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_blocking<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let owner = Owner::current().unwrap_or_default();
    let profile = TermProfile::current();
    let palette = ColorPalette::current();

    tokio::task::spawn_blocking(move || {
        profile.set_local();
        palette.set_local();
        owner.with(f)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_thread<F, T>(f: F) -> thread::JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let owner = Owner::current().unwrap_or_default();
    let profile = TermProfile::current();
    let palette = ColorPalette::current();

    thread::spawn(move || {
        profile.set_local();
        palette.set_local();
        owner.with(f)
    })
}

pub async fn tick() {
    Executor::tick().await
}

pub fn delay<F>(duration: Duration, f: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(async move {
        wasm_compat::futures::sleep(duration).await;
        f.await;
    });
}
