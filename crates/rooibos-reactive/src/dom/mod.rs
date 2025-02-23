use std::future::Future;
use std::io;

pub use children::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use focus::*;
pub use into_view::*;
use ratatui::backend::WindowSize;
pub use renderer::*;
pub use rooibos_dom::{
    DomNodeRepr, MeasureNode, RenderNode, clear_focus, delay, dom_update_receiver, events,
    focus_id, focus_next, focus_prev, line, render_terminal, root, set_pixel_size,
    set_supports_keyboard_enhancement, span, text, try_focus_id, widgets,
};
use rooibos_dom::{render_dom, with_nodes, with_nodes_mut};

mod children;
pub mod div;
mod dom_node;
mod dom_widget;
pub mod flex_node;
mod focus;
mod into_view;
pub mod layout;
mod renderer;

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

pub fn after_render_async(fut: impl Future<Output = ()> + 'static) {
    wasm_compat::futures::spawn_local(fut)
}

pub fn after_render(f: impl FnOnce() + 'static) {
    wasm_compat::futures::spawn_local(async move {
        any_spawner::Executor::tick().await;
        f();
    })
}
