use std::cell::LazyCell;
use std::future::Future;

pub use children::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use focus::*;
pub use into_view::*;
use ratatui::layout::Rect;
use reactive_graph::signal::{ReadSignal, signal};
use reactive_graph::traits::Set as _;
pub use renderer::*;
use rooibos_dom::{on_window_focus_changed, with_nodes, with_nodes_mut};

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
    static WINDOW_SIZE_SIGNAL: LazyCell<ReadSignal<Rect>> = LazyCell::new(|| {
        let (window_size, set_window_size) = signal(Rect::default());
        with_nodes_mut(|nodes| nodes.on_window_size_change(move |size| set_window_size.set(size)));
        window_size
    });

    static WINDOW_FOCUSED_SIGNAL: LazyCell<ReadSignal<bool>> = LazyCell::new(|| {
        let (window_focused, set_window_focused) = signal(true);
        on_window_focus_changed(move |focused| {
            set_window_focused.set(focused);
        });
        window_focused
    });
}

pub fn mount<F, M>(f: F)
where
    F: FnOnce() -> M + 'static,
    M: Render,
    <M as Render>::DomState: 'static,
{
    let node = f().build();
    rooibos_dom::mount(node);
}

pub fn use_window_size() -> ReadSignal<Rect> {
    WINDOW_SIZE_SIGNAL.with(move |s| **s)
}

pub fn use_window_focus() -> ReadSignal<bool> {
    WINDOW_FOCUSED_SIGNAL.with(move |s| **s)
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
