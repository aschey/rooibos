use std::cell::{Ref, RefCell, RefMut};
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};

use ratatui::Frame;
use reactive_graph::signal::ReadSignal;
use slotmap::SlotMap;
use tachys::prelude::*;
use tachys::renderer::CastFrom;
use tokio::sync::watch;

use self::dom_state::DomState;
use crate::make_dom_widget;

mod component;
mod document_fragment;
mod dom_node;
mod dom_state;
mod dom_widget;
mod dyn_child;
mod element;
mod for_each;
mod unit;
mod view;

pub use component::*;
pub use document_fragment::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use element::*;
pub use for_each::*;
pub use view::*;

pub trait Render: tachys::view::Render<RooibosDom, State = DomNode> {}

impl<T> Render for T where T: tachys::view::Render<RooibosDom, State = DomNode> {}

// Reference for focus impl https://github.com/reactjs/rfcs/pull/109/files

static NODE_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) fn next_node_id() -> u32 {
    NODE_ID.fetch_add(1, Ordering::SeqCst)
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

// pub trait ToDomNode {
//     fn to_dom_node(&self) -> DomNode;
// }

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
        DomNode::from_fragment(DocumentFragment::widget(make_dom_widget(
            "text",
            text.to_owned(),
        )))
    }

    fn create_placeholder() -> Self::Placeholder {
        DomNode::from_fragment(DocumentFragment::transparent(""))
    }

    fn set_text(node: &Self::Text, text: &str) {
        replace_child(node.key(), &Self::create_text_node(text));
    }

    fn set_attribute(node: &Self::Element, name: &str, value: &str) {
        // todo!()
    }

    fn remove_attribute(node: &Self::Element, name: &str) {
        // todo!()
    }

    fn insert_node(parent: &Self::Element, new_child: &Self::Node, marker: Option<&Self::Node>) {
        mount_child(MountKind::Append(parent), new_child);
    }

    fn remove_node(parent: &Self::Element, child: &Self::Node) -> Option<Self::Node> {
        unmount_child(child.key());
        Some(child.clone())
    }

    fn clear_children(parent: &Self::Element) {
        todo!()
    }

    fn remove(node: &Self::Node) {
        unmount_child(node.key());
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn first_child(node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn next_sibling(node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn log_node(node: &Self::Node) {
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
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        mount_child(MountKind::Append(parent), self);
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

fn mount_child(kind: MountKind, child: &DomNode) -> DomNodeKey {
    // let child = child.to_dom_node();
    // let child = child.build();
    match kind {
        MountKind::Append(node) => {
            node.append_child(&child);
        }
        MountKind::Before(node) => {
            node.before(&child);
        }
    }
    child.key()
}

fn cleanup_removed_nodes(
    node: &DomNodeKey,
    nodes: &mut RefMut<'_, SlotMap<DomNodeKey, DomNodeInner>>,
) {
    let children = nodes[*node].children.clone();
    for child in children {
        cleanup_removed_nodes(&child, nodes);
    }
    nodes.remove(*node);
}

fn disconnect_child(child: DomNodeKey) {
    DOM_NODES.with(|d| {
        let mut d = d.borrow_mut();
        let child_node = &d[child];
        if let Some(parent) = child_node.parent {
            let child_pos = d[parent].children.iter().position(|c| c == &child).unwrap();
            d[parent].children.remove(child_pos);
        }
        d[child].parent = None;
    });
}

fn replace_child(current: DomNodeKey, new: &DomNode) {
    let parent = DOM_NODES.with(|d| {
        let d = d.borrow();
        let current_node = &d[current];
        current_node.parent
    });
    disconnect_child(current);
    if let Some(parent) = parent {
        mount_child(MountKind::Append(&DomNode::from_key(parent)), new);
    }
}

fn unmount_child(child: DomNodeKey) {
    DOM_NODES.with(|d| {
        let mut d = d.borrow_mut();
        let child_node = &d[child];
        if let Some(parent) = child_node.parent {
            let child_pos = d[parent].children.iter().position(|c| c == &child).unwrap();
            d[parent].children.remove(child_pos);
        }

        cleanup_removed_nodes(&child, &mut d);
    });
}

pub fn print_dom<W: io::Write>(writer: &mut W, include_transparent: bool) -> io::Result<()> {
    DOM_ROOT.with(|dom| {
        DOM_NODES.with(|nodes| {
            let dom = dom.borrow();
            let nodes = nodes.borrow();
            let root = &nodes[dom.as_ref().unwrap().key()];
            if !include_transparent && root.node_type == NodeType::Transparent {
                for (key, _) in &root.resolve_children(&nodes) {
                    print_dom_inner(writer, &nodes, *key, "", include_transparent)?;
                }
            } else {
                print_dom_inner(
                    writer,
                    &nodes,
                    dom.as_ref().unwrap().key(),
                    "",
                    include_transparent,
                )?;
            }

            Ok(())
        })
    })
}

fn print_dom_inner<W: io::Write>(
    writer: &mut W,
    dom_ref: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    key: DomNodeKey,
    indent: &str,
    include_transparent: bool,
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
    write!(writer, " constraint={}", node.constraint)?;

    writeln!(writer, ">")?;
    if let Some(children) = children {
        writeln!(writer, "{indent}  {children}")?;
    }
    let child_indent = format!("{indent}  ");
    if include_transparent {
        for key in &node.children {
            print_dom_inner(writer, dom_ref, *key, &child_indent, include_transparent)?;
        }
    } else {
        for (key, _) in &node.resolve_children(dom_ref) {
            print_dom_inner(writer, dom_ref, *key, &child_indent, include_transparent)?;
        }
    }

    writeln!(writer, "{indent}</{node_name}>")?;

    Ok(())
}

pub(crate) fn notify() {
    DOM_UPDATE_TX.with(|tx| tx.borrow().send(()).ok());
}

pub fn mount<F, M>(f: F, tx: watch::Sender<()>)
where
    F: FnOnce() -> M + 'static,
    M: Render<State = DomNode>,
{
    let node = f().build();
    DOM_ROOT.with(|d| *d.borrow_mut() = Some(node));
    DOM_UPDATE_TX.with(|d| *d.borrow_mut() = tx);
}

pub fn unmount() {
    DOM_ROOT.with(|d| *d.borrow_mut() = None);
    DOM_STATE.with(|d| *d.borrow_mut() = None);
    DOM_NODES.with(|d| (*d.borrow_mut()).clear());
}

pub fn render_dom(frame: &mut Frame) {
    DOM_ROOT.with(|d| d.borrow().as_ref().unwrap().render(frame, frame.size()));
}

pub fn focus(id: impl Into<NodeId>) {
    let id = id.into();
    let node = DOM_NODES.with(|d| {
        d.borrow().iter().find_map(|(k, v)| {
            if v.id.as_ref() == Some(&id) {
                Some(k)
            } else {
                None
            }
        })
    });
    if let Some(node) = node {
        with_state_mut(|state| {
            DOM_NODES.with(|nodes| {
                state.set_focused(Some(node), &nodes.borrow()[node]);
            });
        });
    }
}

pub fn focused_node() -> ReadSignal<Option<NodeId>> {
    with_state(|d| d.focused())
}

pub fn focus_id(id: impl Into<NodeId>) {
    let id = id.into();
    DOM_NODES.with(|nodes| {
        let nodes = nodes.borrow();
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
                state.set_focused(Some(found_node), &nodes[found_node]);
            });
        }
    });
}

pub fn focus_next() {
    DOM_NODES.with(|nodes| {
        let nodes = nodes.borrow();
        with_state_mut(|state| {
            if let Some(focused) = state.focused_key() {
                let current_focused = state
                    .focusable_nodes()
                    .iter()
                    .position(|n| n == &focused)
                    .unwrap();

                if current_focused < state.focusable_nodes().len() - 1 {
                    let next = state.focusable_nodes()[current_focused + 1];
                    state.set_focused(Some(next), &nodes[next]);
                } else {
                    let next = state.focusable_nodes()[0];
                    state.set_focused(Some(next), &nodes[next]);
                }
            } else {
                let next = state.focusable_nodes()[0];
                state.set_focused(Some(next), &nodes[next]);
            }
        });
    });
}

pub fn focus_prev() {
    DOM_NODES.with(|nodes| {
        let nodes = nodes.borrow();
        with_state_mut(|state| {
            if let Some(focused) = state.focused_key() {
                let current_focused = state
                    .focusable_nodes()
                    .iter()
                    .position(|n| n == &focused)
                    .unwrap();
                if current_focused > 0 {
                    let prev = state.focusable_nodes()[current_focused - 1];
                    state.set_focused(Some(prev), &nodes[prev]);
                } else {
                    let prev = state.focusable_nodes()[state.focusable_nodes().len() - 1];
                    state.set_focused(Some(prev), &nodes[prev]);
                }
            } else {
                let prev = state.focusable_nodes()[state.focusable_nodes().len() - 1];
                state.set_focused(Some(prev), &nodes[prev]);
            }
        });
    });
}
