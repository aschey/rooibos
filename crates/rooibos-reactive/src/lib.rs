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

use any_spawner::Executor;
pub use error_boundary::*;
pub use for_loop::*;
pub use provider::*;
#[doc(hidden)]
pub use reactive_graph as __reactive;
use reactive_graph::owner::Owner;
use reactive_graph::signal::signal;
use reactive_graph::traits::{Get, Set};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::events::{EventData, NodeState};
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

#[derive(Clone)]
pub struct StateProp<T> {
    pub focused: Arc<dyn Fn(T) -> T + Send + Sync>,
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

    fn apply_state(&self, node_state: NodeState) -> T {
        let mut value = self.normal.clone();
        if node_state.intersects(NodeState::HOVERED) {
            value = (self.hovered)(value);
        }
        if node_state.intersects(NodeState::FOCUSED) {
            value = (self.focused)(value);
        }
        if node_state.intersects(NodeState::DISABLED) {
            value = (self.disabled)(value);
        }
        value
    }
}

pub fn use_state_prop<T>(
    state_prop: impl Into<Signal<StateProp<T>>>,
) -> (Signal<T>, impl Fn(NodeState, EventData))
where
    T: Clone + Send + Sync + 'static,
{
    let (widget_state, set_widget_state) = signal(NodeState::empty());
    let state_prop = state_prop.into();
    let prop = derive_signal!({
        let state_prop = state_prop.get();
        state_prop.apply_state(widget_state.get())
    });
    (prop, move |state: NodeState, _| set_widget_state.set(state))
}
