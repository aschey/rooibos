use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub use any_view::*;
pub use children::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use focus::*;
pub use into_view::*;
pub use node_tree::*;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use reactive_graph::signal::ReadSignal;
pub use renderer::*;
use terminput::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use tokio::sync::watch;
use tracing::error;

use crate::{text, EventData, MouseEventFn};

mod any_view;
mod children;
pub mod div;
mod dom_node;
mod dom_widget;
pub mod flex_node;
mod focus;
mod into_view;
pub mod layout;
mod node_tree;
mod renderer;

// Reference for focus impl https://github.com/reactjs/rfcs/pull/109/files

static NODE_ID: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static EVENT_EMITTER: RefCell<EventEmitter> = RefCell::new(EventEmitter::new())
}

pub(crate) fn next_node_id() -> u32 {
    NODE_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct DomUpdateReceiver(watch::Receiver<()>);

impl DomUpdateReceiver {
    pub async fn changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.0.changed().await
    }
}

thread_local! {
    static DOM_UPDATE_TX: RefCell<watch::Sender<()>> = {
        let (tx, _) = watch::channel(());
        RefCell::new(tx)
    };
    static PRINT_DOM: AtomicBool = const { AtomicBool::new(false) };
    static PENDING_RESIZE: AtomicBool = const { AtomicBool::new(true) };
}

pub fn dom_update_receiver() -> DomUpdateReceiver {
    let rx = DOM_UPDATE_TX.with(|d| d.borrow().subscribe());
    DomUpdateReceiver(rx)
}

fn cleanup_removed_nodes(node: &DomNodeKey, remove: bool) {
    let children = with_nodes_mut(|nodes| {
        nodes.unset_state(node);
        nodes[*node].children.clone()
    });

    for child in children {
        cleanup_removed_nodes(&child, remove);
    }
    if remove {
        let removed = with_nodes_mut(|nodes| nodes.remove(*node));
        // We need to make sure we drop the removed node after we release the borrow
        // because the drop impl needs to borrow the node list as well
        drop(removed);
    }
}

fn clear_children(parent: DomNodeKey) {
    let children = with_nodes(|nodes| nodes[parent].children.clone());
    for child in children {
        unmount_child(child, true);
    }
}

fn unmount_child(child: DomNodeKey, cleanup: bool) {
    with_nodes_mut(|nodes| {
        nodes.unmount_child(child);
    });

    cleanup_removed_nodes(&child, cleanup);
    refresh_dom();
}

pub fn print_dom() -> Paragraph<'static> {
    let lines = with_nodes(|nodes| print_dom_inner(nodes, nodes.root(0).as_dom_node().key(), ""));
    Paragraph::new(lines.clone()).wrap(Wrap { trim: false })
}

pub fn root() -> DomNodeRepr {
    with_nodes(|nodes| {
        let root = nodes.root(0).as_dom_node().key();
        let node = &nodes[root];
        DomNodeRepr::from_node(root, node)
    })
}

fn print_dom_inner(dom_ref: &NodeTree, key: DomNodeKey, indent: &str) -> Vec<Line<'static>> {
    let node = &dom_ref[key];
    let NodeTypeStructure { name, attrs } = node.node_type.structure();
    let node_name = node.name.clone();
    if node_name == "Placeholder" {
        return vec![];
    }

    let mut line = format!(
        "{indent}<{node_name} type={name} key={key:?} parent={:?}",
        node.parent
    );

    if let Some(attrs) = attrs {
        line += &format!(" {attrs}");
    }
    line += &format!(" display={:?}", dom_ref.style(key).display);
    line += &format!(" layout={:?}>", dom_ref.rect(key));
    // line += &format!(" constraint={}>", node.constraint.borrow().clone());

    let mut lines = vec![crate::line!(line)];

    let child_indent = format!("{indent}  ");

    for key in &node.children {
        lines.append(&mut print_dom_inner(dom_ref, *key, &child_indent));
    }

    lines.push(crate::line!("{indent}</{node_name}>"));

    lines
}

pub fn refresh_dom() {
    let _ = DOM_UPDATE_TX.with(|tx| {
        tx.borrow()
            .send(())
            .inspect_err(|e| error!("error sending DOM update: {e:?}"))
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
        n.set_root(0, Box::new(node));
    });
}

pub fn unmount() {
    // reset_state();
    with_nodes_mut(|d| *d = NodeTree::new());
}

pub fn render_dom(buf: &mut Buffer) {
    if PENDING_RESIZE.with(|p| p.swap(false, Ordering::Relaxed)) {
        with_nodes(|nodes| nodes.set_window_size(buf.area));
    }

    if PRINT_DOM.with(|p| p.load(Ordering::Relaxed)) {
        print_dom().render_ref(buf.area, buf);
    } else {
        with_nodes_mut(|nodes| {
            nodes.recompute_layout(buf.area);
            nodes.clear_focusables();
        });

        let roots = with_nodes(|nodes| nodes.roots_asc());
        for root in roots {
            root.render(buf, buf.area);
        }
    }
}

pub fn focus(id: impl Into<NodeId>) {
    let id = id.into();
    with_nodes_mut(|nodes| {
        let node = nodes.iter_nodes().find_map(|(k, v)| {
            if v.inner.id.as_ref() == Some(&id) {
                Some(k)
            } else {
                None
            }
        });
        if let Some(node) = node {
            nodes.set_focused(node);
        }
    });
}

pub fn use_focused_node() -> ReadSignal<Option<NodeId>> {
    with_nodes(|nodes| nodes.focused())
}

pub fn use_window_size() -> ReadSignal<Rect> {
    with_nodes(|nodes| nodes.window_size())
}

struct ClickEvent {
    on_click: Option<MouseEventFn>,
    rect: Rect,
    key: DomNodeKey,
}

pub fn send_event(event: Event) {
    EVENT_EMITTER.with(|e| e.borrow_mut().send_event(event))
}

pub(crate) fn reset_mouse_position() {
    EVENT_EMITTER.with(|e| e.borrow_mut().reset_mouse_position())
}

struct EventEmitter {
    last_mouse_position: MouseEvent,
}

impl EventEmitter {
    fn new() -> Self {
        Self {
            last_mouse_position: MouseEvent {
                kind: MouseEventKind::Moved,
                column: u16::MAX,
                row: u16::MAX,
                modifiers: KeyModifiers::empty(),
            },
        }
    }

    fn reset_mouse_position(&mut self) {
        self.send_event(Event::Mouse(self.last_mouse_position))
    }

    fn send_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => {
                if key_event.code == KeyCode::Tab && key_event.kind == KeyEventKind::Press {
                    focus_next();
                } else if key_event.code == KeyCode::BackTab
                    && key_event.kind == KeyEventKind::Press
                {
                    focus_prev();
                } else if key_event.code == KeyCode::Char('x')
                    && key_event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    PRINT_DOM.with(|p| p.swap(!p.load(Ordering::Relaxed), Ordering::Relaxed));
                    refresh_dom();
                } else if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
                    match key_event.kind {
                        KeyEventKind::Press | KeyEventKind::Repeat => {
                            let (rect, mut on_key_down) = with_nodes(|nodes| {
                                (
                                    *nodes[key].rect.borrow(),
                                    nodes[key].event_handlers.on_key_down.clone(),
                                )
                            });
                            if let Some(on_key_down) = &mut on_key_down {
                                #[cfg(debug_assertions)]
                                let _guard =
                                    reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                                on_key_down.borrow_mut()(key_event, EventData { rect });
                            }
                        }
                        KeyEventKind::Release => {
                            let (rect, mut on_key_up) = with_nodes(|nodes| {
                                (
                                    *nodes[key].rect.borrow(),
                                    nodes[key].event_handlers.on_key_up.clone(),
                                )
                            });
                            if let Some(on_key_up) = &mut on_key_up {
                                #[cfg(debug_assertions)]
                                let _guard =
                                    reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                                on_key_up.borrow_mut()(key_event, EventData { rect });
                            }
                        }
                    }
                }
            }
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::Down(_) => {
                    let current = with_nodes(|nodes| {
                        let current: Rc<RefCell<Option<ClickEvent>>> = Rc::new(RefCell::new(None));
                        for root in nodes.roots_desc() {
                            let found = root.key().traverse(
                                |key, inner| {
                                    if inner.disabled {
                                        return None;
                                    }

                                    let rect = inner.rect.borrow();
                                    if rect.contains(Position {
                                        x: mouse_event.column,
                                        y: mouse_event.row,
                                    }) {
                                        if inner.focusable
                                            || inner.event_handlers.on_click.is_some()
                                        {
                                            *current.borrow_mut() = Some(ClickEvent {
                                                on_click: inner.event_handlers.on_click.clone(),
                                                rect: *rect,
                                                key,
                                            });
                                        }
                                        if inner.event_handlers.on_click.is_some() {
                                            return Some(());
                                        }
                                    }
                                    None
                                },
                                true,
                            );
                            if !found.is_empty() {
                                break;
                            }
                        }

                        current
                    });
                    let current = current.borrow();
                    if let Some(ClickEvent {
                        on_click,
                        rect,
                        key,
                    }) = current.as_ref()
                    {
                        with_nodes_mut(|nodes| {
                            let set_focus =
                                nodes.focused_key() != Some(*key) && nodes[*key].focusable;
                            if set_focus {
                                nodes.set_focused(*key);
                            }
                        });

                        if let Some(on_click) = on_click {
                            #[cfg(debug_assertions)]
                            let _guard =
                                reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                            on_click.borrow_mut()(mouse_event, EventData { rect: *rect });
                        }
                    }
                }
                MouseEventKind::Up(_) => {}
                MouseEventKind::Drag(_) => {}
                MouseEventKind::Moved => {
                    self.last_mouse_position = mouse_event;
                    let current = with_nodes(|nodes| {
                        for root in nodes.roots_desc() {
                            let found = root.key().traverse(
                                |key, inner| {
                                    if inner.disabled {
                                        return None;
                                    }

                                    if inner.rect.borrow().contains(Position {
                                        x: mouse_event.column,
                                        y: mouse_event.row,
                                    }) && inner.focusable
                                    {
                                        Some(key)
                                    } else {
                                        None
                                    }
                                },
                                true,
                            );
                            if let Some(found) = found.first() {
                                return Some(*found);
                            }
                        }
                        None
                    });

                    if let Some(current) = current {
                        with_nodes_mut(|nodes| {
                            let set_focus = nodes.hovered_key() != Some(current);
                            if set_focus {
                                nodes.set_hovered(current);
                            }
                        });
                    } else {
                        with_nodes_mut(|nodes| {
                            nodes.remove_hovered();
                        });
                    }
                }
                MouseEventKind::ScrollDown => {}
                MouseEventKind::ScrollUp => {}
                MouseEventKind::ScrollLeft => {}
                MouseEventKind::ScrollRight => {}
            },
            Event::Paste(val) => {
                if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
                    let (rect, on_paste) = with_nodes(|nodes| {
                        (
                            *nodes[key].rect.borrow(),
                            nodes[key].event_handlers.on_paste.clone(),
                        )
                    });
                    if let Some(on_paste) = on_paste {
                        #[cfg(debug_assertions)]
                        let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                        on_paste.borrow_mut()(val, EventData { rect });
                    }
                }
            }
            Event::Resize(_, _) => {
                PENDING_RESIZE.with(|p| p.store(true, Ordering::Relaxed));
                refresh_dom();
            }
        };
    }
}

pub fn focus_id(id: impl Into<NodeId>) {
    let id = id.into();
    with_nodes_mut(|nodes| {
        let found_node = nodes.iter_nodes().find_map(|(key, node)| {
            if let Some(current_id) = &node.inner.id {
                if &id == current_id {
                    return Some(key);
                }
            }
            None
        });
        if let Some(found_node) = found_node {
            nodes.set_focused(found_node);
        }
    });
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
