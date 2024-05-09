use std::cell::{Ref, RefCell, RefMut};
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};

use ratatui::layout::Position;
use ratatui::text::Text;
use ratatui::Frame;
use reactive_graph::signal::ReadSignal;
use slotmap::SlotMap;
use tachys::prelude::*;
use tachys::renderer::CastFrom;
use tokio::sync::watch;

use self::dom_state::DomState;
use crate::{make_dom_widget, Event, KeyCode, KeyEventKind, KeyModifiers, NewExt};

mod any_view;
mod children;
mod document_fragment;
mod dom_node;
mod dom_state;
mod dom_widget;
mod element;
mod focus;
mod into_view;

pub use any_view::*;
pub use children::*;
pub use document_fragment::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use element::*;
pub use focus::*;
pub use into_view::*;

pub trait RenderAny: tachys::view::Render<RooibosDom> {}

impl<T> RenderAny for T where T: tachys::view::Render<RooibosDom> {}

pub trait Render: tachys::view::Render<RooibosDom, State = DomNode> {}

impl<T> Render for T where T: tachys::view::Render<RooibosDom, State = DomNode> {}

// Reference for focus impl https://github.com/reactjs/rfcs/pull/109/files

static NODE_ID: AtomicU32 = AtomicU32::new(1);

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
    static DOM_ROOT: RefCell<Option<DomNode>> = const { RefCell::new(None) };
    static DOM_NODES: RefCell<SlotMap<DomNodeKey, DomNodeInner>> =
        RefCell::new(SlotMap::<DomNodeKey, DomNodeInner>::default());
    static DOM_STATE: RefCell<Option<DomState>> = RefCell::new(Some(Default::default()));
    static DOM_UPDATE_TX: RefCell<watch::Sender<()>> = {
        let (tx, _) = watch::channel(());
        RefCell::new(tx)
    };
}

pub fn dom_update_receiver() -> DomUpdateReceiver {
    let (tx, rx) = watch::channel(());
    DOM_UPDATE_TX.with(|d| *d.borrow_mut() = tx);
    DomUpdateReceiver(rx)
}

fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&DomState) -> R,
{
    DOM_STATE.with(|s| {
        let s = s.borrow();
        if let Some(s) = s.as_ref() {
            return f(s);
        }
        panic!("state deallocated")
    })
}

fn with_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut DomState) -> R,
{
    DOM_STATE.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(s) = s.as_mut() {
            return f(s);
        }
        panic!("state deallocated")
    })
}

fn with_nodes<F, R>(f: F) -> R
where
    F: FnOnce(Ref<SlotMap<DomNodeKey, DomNodeInner>>) -> R,
{
    DOM_NODES.with(|n| {
        let n = n.borrow();
        f(n)
    })
}

fn with_nodes_mut<F, R>(f: F) -> R
where
    F: FnOnce(RefMut<SlotMap<DomNodeKey, DomNodeInner>>) -> R,
{
    DOM_NODES.with(|n| {
        let n = n.borrow_mut();
        f(n)
    })
}

fn with_root<F, R>(f: F) -> R
where
    F: FnOnce(Ref<Option<DomNode>>) -> R,
{
    DOM_ROOT.with(|n| {
        let n = n.borrow();
        f(n)
    })
}

fn with_root_mut<F, R>(f: F) -> R
where
    F: FnOnce(RefMut<Option<DomNode>>) -> R,
{
    DOM_ROOT.with(|n| {
        let n = n.borrow_mut();
        f(n)
    })
}

#[derive(Debug)]
pub struct RooibosDom;

impl Renderer for RooibosDom {
    type Node = DomNode;

    type Element = DomNode;

    type Text = DomNode;

    type Placeholder = DomNode;

    fn intern(text: &str) -> &str {
        text
    }

    fn create_text_node(text: &str) -> Self::Text {
        make_dom_widget("text", Text::new(text.to_owned())).inner()
    }

    fn create_placeholder() -> Self::Placeholder {
        make_dom_widget("placeholder", Text::new("")).inner()
    }

    fn set_text(node: &Self::Text, text: &str) {
        let text = text.to_string();
        node.replace_fragment(DocumentFragment::widget(DomWidgetNode::new(
            "text",
            move || {
                let text = text.clone();
                move |frame, rect| {
                    frame.render_widget(&text, rect);
                }
            },
        )));
    }

    fn set_attribute(_node: &Self::Element, _name: &str, _value: &str) {
        unimplemented!()
    }

    fn remove_attribute(_node: &Self::Element, _name: &str) {
        unimplemented!()
    }

    fn insert_node(parent: &Self::Element, new_child: &Self::Node, _marker: Option<&Self::Node>) {
        mount_child(parent, new_child);
    }

    fn remove_node(_parent: &Self::Element, child: &Self::Node) -> Option<Self::Node> {
        unmount_child(child.key());
        Some(child.clone())
    }

    fn clear_children(_parent: &Self::Element) {
        todo!()
    }

    fn remove(node: &Self::Node) {
        unmount_child(node.key());
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        node.get_parent()
    }

    fn first_child(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn next_sibling(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn log_node(_node: &Self::Node) {
        todo!()
    }
}

impl CastFrom<DomNode> for DomNode {
    fn cast_from(source: DomNode) -> Option<Self> {
        Some(source)
    }
}

impl AsRef<DomNode> for DomNode {
    fn as_ref(&self) -> &DomNode {
        self
    }
}

impl Mountable<RooibosDom> for DomNode {
    fn unmount(&mut self) {
        unmount_child(self.key())
    }

    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        _marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        mount_child(parent, self);
    }

    fn insert_before_this(
        &self,
        parent: &<RooibosDom as Renderer>::Element,
        child: &mut dyn Mountable<RooibosDom>,
    ) -> bool {
        child.mount(parent, Some(self));
        true
    }
}

pub enum MountKind<'a> {
    Before(&'a DomNode),
    Append(&'a DomNode),
}

fn mount_child(parent: &DomNode, child: &DomNode) -> DomNodeKey {
    parent.append_child(child);
    notify();
    child.key()
}

fn cleanup_removed_nodes(
    node: &DomNodeKey,
    nodes: &mut RefMut<'_, SlotMap<DomNodeKey, DomNodeInner>>,
) {
    with_state_mut(|s| {
        s.remove_focusable(node);
    });
    let children = nodes[*node].children.clone();
    for child in children {
        cleanup_removed_nodes(&child, nodes);
    }
    nodes.remove(*node);
}

// fn disconnect_child(child: DomNodeKey) {
//     DOM_NODES.with(|d| {
//         let mut d = d.borrow_mut();
//         let child_node = &d[child];
//         if let Some(parent) = child_node.parent {
//             let child_pos = d[parent].children.iter().position(|c| c == &child).unwrap();
//             d[parent].children.remove(child_pos);
//         }
//         d[child].parent = None;
//     });
// }

// fn replace_child(current: DomNodeKey, new: &DomNode) {
//     let parent = DOM_NODES.with(|d| {
//         let d = d.borrow();
//         let current_node = &d[current];
//         current_node.parent
//     });
//     disconnect_child(current);
//     if let Some(parent) = parent {
//         mount_child(&DomNode::from_key(parent), new);
//     }
// }

fn unmount_child(child: DomNodeKey) {
    with_nodes_mut(|mut d| {
        let child_node = &d[child];
        if let Some(parent) = child_node.parent {
            let child_pos = d[parent].children.iter().position(|c| c == &child).unwrap();
            d[parent].children.remove(child_pos);
        }

        cleanup_removed_nodes(&child, &mut d);
    });
    notify();
}

pub fn print_dom<W: io::Write>(writer: &mut W) -> io::Result<()> {
    with_root(|dom| {
        with_nodes(|nodes| {
            print_dom_inner(writer, &nodes, dom.as_ref().unwrap().key(), "")?;
            Ok(())
        })
    })
}

fn print_dom_inner<W: io::Write>(
    writer: &mut W,
    dom_ref: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    key: DomNodeKey,
    indent: &str,
) -> io::Result<()> {
    let node = &dom_ref[key];
    let NodeTypeStructure {
        name,
        attrs,
        children,
    } = node.node_type.structure();
    let node_name = node.name.clone();
    write!(
        writer,
        "{indent}<{node_name} type={name} key={key:?} parent={:?}",
        node.parent
    )?;
    if let Some(attrs) = attrs {
        write!(writer, " {attrs}")?;
    }
    write!(writer, " constraint={}", node.constraint.borrow().clone())?;

    writeln!(writer, ">")?;
    if let Some(children) = children {
        writeln!(writer, "{indent}  {children}")?;
    }
    let child_indent = format!("{indent}  ");
    for key in &node.children {
        print_dom_inner(writer, dom_ref, *key, &child_indent)?;
    }

    writeln!(writer, "{indent}</{node_name}>")?;

    Ok(())
}

pub(crate) fn notify() {
    DOM_UPDATE_TX.with(|tx| tx.borrow().send(()).ok());
}

pub fn mount<F, M>(f: F)
where
    F: FnOnce() -> M + 'static,
    M: Render<State = DomNode>,
{
    let node = f().build();
    with_root_mut(|mut d| *d = Some(node));
}

pub fn unmount() {
    with_root_mut(|mut d| *d = None);
    DOM_STATE.with(|d| *d.borrow_mut() = None);
    with_nodes_mut(|mut d| (*d).clear());
}

pub fn render_dom(frame: &mut Frame) {
    with_root(|d| d.as_ref().unwrap().render(frame, frame.size()));
}

pub fn focus(id: impl Into<NodeId>) {
    let id = id.into();
    let node = with_nodes(|d| {
        d.iter().find_map(|(k, v)| {
            if v.id.as_ref() == Some(&id) {
                Some(k)
            } else {
                None
            }
        })
    });
    if let Some(node) = node {
        with_state_mut(|state| {
            with_nodes_mut(|mut nodes| {
                state.set_focused(node, &mut nodes);
            });
        });
    }
}

pub fn focused_node() -> ReadSignal<Option<NodeId>> {
    with_state(|d| d.focused())
}

enum Focus {
    Next,
    Prev,
}

pub fn send_event(event: Event) {
    let focus = with_state_mut(|state| {
        with_nodes_mut(|mut nodes| {
            match event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Tab && key_event.kind == KeyEventKind::Press {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            return Some(Focus::Prev);
                        } else {
                            return Some(Focus::Next);
                        }
                    } else if let Some(key) = state.focused_key() {
                        match key_event.kind {
                            KeyEventKind::Press | KeyEventKind::Repeat => {
                                if let Some(on_key_down) =
                                    &mut nodes[key].event_handlers.on_key_down
                                {
                                    on_key_down.borrow_mut()(key_event);
                                }
                            }
                            KeyEventKind::Release => {
                                if let Some(on_key_up) = &mut nodes[key].event_handlers.on_key_up {
                                    on_key_up.borrow_mut()(key_event);
                                }
                            }
                        }
                    }
                }

                Event::FocusGained => todo!(),
                Event::FocusLost => todo!(),
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    crate::MouseEventKind::Down(_) => {
                        let current = nodes.keys().find(|k| {
                            nodes[*k].rect.borrow().contains(Position {
                                x: mouse_event.column,
                                y: mouse_event.row,
                            }) && nodes[*k].event_handlers.on_click.is_some()
                        });
                        if let Some(current) = current {
                            if let Some(on_click) = &mut nodes[current].event_handlers.on_click {
                                on_click.borrow_mut()(mouse_event);
                            }
                        }
                    }
                    crate::MouseEventKind::Up(_) => {}
                    crate::MouseEventKind::Drag(_) => {}
                    crate::MouseEventKind::Moved => {
                        let current = nodes.keys().find(|k| {
                            nodes[*k].rect.borrow().contains(Position {
                                x: mouse_event.column,
                                y: mouse_event.row,
                            }) && *nodes[*k].focusable.borrow()
                        });
                        if let Some(current) = current {
                            if state.focused_key() != Some(current) {
                                state.set_focused(current, &mut nodes);
                            }
                        }
                    }
                    crate::MouseEventKind::ScrollDown => {}
                    crate::MouseEventKind::ScrollUp => {}
                    crate::MouseEventKind::ScrollLeft => {}
                    crate::MouseEventKind::ScrollRight => {}
                },
                Event::Paste(_) => todo!(),
                Event::Resize(_, _) => todo!(),
            };
            None
        })
    });

    match focus {
        Some(Focus::Next) => {
            focus_next();
        }
        Some(Focus::Prev) => {
            focus_prev();
        }
        None => {}
    }
}

pub fn focus_id(id: impl Into<NodeId>) {
    let id = id.into();
    with_nodes_mut(|mut nodes| {
        let found_node = nodes.iter().find_map(|(key, node)| {
            if let Some(current_id) = &node.id {
                if &id == current_id {
                    return Some(key);
                }
            }
            None
        });
        if let Some(found_node) = found_node {
            with_state_mut(|state| {
                state.set_focused(found_node, &mut nodes);
            });
        }
    });
}

pub fn focus_next() {
    with_nodes_mut(|mut nodes| {
        with_state_mut(|state| {
            let focusable_nodes = state.focusable_nodes();
            if let Some(focused) = state.focused_key() {
                let current_focused = focusable_nodes.iter().position(|n| n == &focused).unwrap();

                if current_focused < focusable_nodes.len() - 1 {
                    let next = focusable_nodes[current_focused + 1];
                    state.set_focused(next, &mut nodes);
                    return;
                }
            }
            if let Some(next) = focusable_nodes.first() {
                state.set_focused(*next, &mut nodes);
            }
        });
    });
}

pub fn focus_prev() {
    with_nodes_mut(|mut nodes| {
        with_state_mut(|state| {
            let focusable_nodes = state.focusable_nodes();
            if let Some(focused) = state.focused_key() {
                let current_focused = focusable_nodes.iter().position(|n| n == &focused).unwrap();
                if current_focused > 0 {
                    let prev = focusable_nodes[current_focused - 1];
                    state.set_focused(prev, &mut nodes);
                    return;
                }
            }
            if let Some(prev) = focusable_nodes.last() {
                state.set_focused(*prev, &mut nodes);
            }
        });
    });
}
