use std::cell::LazyCell;
use std::future::Future;
use std::io;

pub use children::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use focus::*;
pub use into_view::*;
use ratatui::backend::WindowSize;
use ratatui::layout::Rect;
use reactive_graph::signal::{ArcReadSignal, ReadSignal, arc_signal};
use reactive_graph::traits::Set as _;
pub use renderer::*;
pub use rooibos_dom::{
    DomNodeRepr, MeasureNode, RenderNode, clear_focus, delay, dom_update_receiver, events,
    focus_id, focus_next, focus_prev, line, render_terminal, root, set_pixel_size,
    set_supports_keyboard_enhancement, span, text, try_focus_id, widgets,
};
use rooibos_dom::{on_window_focus_changed, render_dom, with_nodes, with_nodes_mut};

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
    static WINDOW_SIZE_SIGNAL: LazyCell<ArcReadSignal<Rect>> = LazyCell::new(|| {
        let (window_size, set_window_size) = arc_signal(Rect::default());
        with_nodes_mut(|nodes| nodes.on_window_size_change(move |size| set_window_size.set(size)));
        window_size
    });

    static WINDOW_FOCUSED_SIGNAL: LazyCell<ArcReadSignal<bool>> = LazyCell::new(|| {
        let (window_focused, set_window_focused) = arc_signal(true);
        on_window_focus_changed(move |focused| {
            set_window_focused.set(focused);
        });
        window_focused
    });
}

pub fn mount<F, M>(f: F, window_size: Option<WindowSize>)
where
    F: FnOnce() -> M + 'static,
    M: Render,
    <M as Render>::DomState: 'static,
{
    let _ = set_pixel_size(window_size);
    let node = f().build();
    rooibos_dom::mount(node);
}

pub fn render_single_frame<F, M, B>(
    f: F,
    terminal: &mut ratatui::Terminal<B>,
) -> Result<(), io::Error>
where
    F: FnOnce() -> M + 'static,
    M: Render,
    <M as Render>::DomState: 'static,
    B: ratatui::backend::Backend + io::Write + 'static,
{
    let window_size = terminal.backend_mut().window_size().ok();

    mount(f, window_size);
    terminal.draw(render_dom)?;
    terminal.backend_mut().write_all(b"\n")?;
    Ok(())
}

pub fn use_window_size() -> ReadSignal<Rect> {
    WINDOW_SIZE_SIGNAL.with(move |s| ReadSignal::from((**s).clone()))
}

pub fn use_window_focus() -> ReadSignal<bool> {
    WINDOW_FOCUSED_SIGNAL.with(move |s| ReadSignal::from((**s).clone()))
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
