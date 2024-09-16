use std::cell::LazyCell;
use std::future::Future;

pub use any_view::*;
pub use children::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use focus::*;
pub use into_view::*;
use ratatui::layout::Rect;
use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::Set as _;
pub use renderer::*;
use rooibos_dom2::{with_nodes, with_nodes_mut};

use crate::text;

mod any_view;
mod children;
pub mod div;
mod dom_node;
mod dom_widget;
pub mod flex_node;
mod focus;
mod into_view;
pub mod layout;
mod renderer;

thread_local! {
    static WINDOW_SIGNAL: LazyCell<ReadSignal<Rect>> = LazyCell::new(|| {
        let (window_size, set_window_size) = signal(Rect::default());
        with_nodes_mut(|nodes| nodes.on_window_size_change(move |size| set_window_size.set(size)));
        window_size
    });
}

pub fn mount<F, M>(f: F)
where
    F: FnOnce() -> M + 'static,
    M: Render,
    <M as Render>::DomState: 'static,
{
    let node = f().build();
    with_nodes_mut(|n| {
        n.set_root(0, node);
    });
}

pub fn use_window_size() -> ReadSignal<Rect> {
    WINDOW_SIGNAL.with(move |s| **s)
}

pub fn after_render_async(fut: impl Future<Output = ()> + 'static) {
    wasm_compat::futures::spawn_local(fut)
}

pub fn after_render(f: impl FnOnce() + 'static) {
    wasm_compat::futures::spawn_local(async move {
        any_spawner::Executor::tick().await;
        f();
    })
}
