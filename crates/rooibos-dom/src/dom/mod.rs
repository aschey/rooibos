use std::cell::{Ref, RefCell, RefMut};
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};

use ratatui::Frame;
use rooibos_reactive::{Disposer, ReadSignal};
use slotmap::SlotMap;

use self::dom_state::DomState;

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

// Reference for focus impl https://github.com/reactjs/rfcs/pull/109/files

static NODE_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) fn next_node_id() -> u32 {
    NODE_ID.fetch_add(1, Ordering::SeqCst)
}

thread_local! {
    static DOM_ROOT: RefCell<Option<DomNode>> = RefCell::new(None);
    static DOM_NODES: RefCell<SlotMap<DomNodeKey, DomNodeInner>> =
        RefCell::new(SlotMap::<DomNodeKey, DomNodeInner>::default());
    static DOM_STATE: RefCell<DomState> = RefCell::new(Default::default());
}

pub enum MountKind<'a> {
    Before(&'a DomNode),
    Append(&'a DomNode),
}

fn mount_child<M: Mountable + std::fmt::Debug>(kind: MountKind, child: &M) -> DomNodeKey {
    let child = child.get_mountable_node();

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

pub fn mount<F, IV>(f: F)
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView,
{
    let node = f().into_view().get_mountable_node();
    DOM_ROOT.with(|d| *d.borrow_mut() = Some(node));
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
        DOM_STATE.with(|state| {
            DOM_NODES.with(|nodes| {
                state
                    .borrow_mut()
                    .set_focused(Some(node), &nodes.borrow()[node]);
            });
        });
    }
}

pub fn focused_node() -> ReadSignal<Option<NodeId>> {
    DOM_STATE.with(|d| d.borrow().focused())
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
            DOM_STATE.with(|state| {
                state
                    .borrow_mut()
                    .set_focused(Some(found_node), &nodes[found_node]);
            });
        }
    });
}

pub fn focus_next() {
    DOM_NODES.with(|nodes| {
        let nodes = nodes.borrow();
        DOM_STATE.with(|state| {
            let mut state = state.borrow_mut();
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
        DOM_STATE.with(|state| {
            let mut state = state.borrow_mut();
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
