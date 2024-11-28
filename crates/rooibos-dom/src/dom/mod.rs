use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::{self, Thread};

pub use dom_node::*;
pub use dom_widget::*;
pub use node_tree::*;
use ratatui::backend::Backend;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use ratatui::{CompletedFrame, Frame, Terminal};
use tokio::sync::{oneshot, watch};
use tokio::task::{LocalEnterGuard, LocalSet};

use crate::NonblockingTerminal;

mod dom_node;
mod dom_widget;

mod node_tree;

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
    static DOM_UPDATE_TX: RefCell<watch::Sender<()>> = {
        let (tx, _) = watch::channel(());
        RefCell::new(tx)
    };
    static PRINT_DOM: AtomicBool = const { AtomicBool::new(false) };
    static PENDING_RESIZE: AtomicBool = const { AtomicBool::new(true) };
    static ON_WINDOW_FOCUS_CHANGE: RefCell<Box<dyn FnMut(bool)>> = {
        RefCell::new(Box::new(|_focused| {}))
    };
}

pub fn on_window_focus_changed<F>(f: F)
where
    F: FnMut(bool) + 'static,
{
    ON_WINDOW_FOCUS_CHANGE.with(|on_change| *on_change.borrow_mut() = Box::new(f));
}

pub(crate) fn trigger_window_focus_changed(focused: bool) {
    ON_WINDOW_FOCUS_CHANGE.with(|on_change| (on_change.borrow_mut())(focused));
}

pub(crate) fn toggle_print_dom() {
    PRINT_DOM.with(|p| p.swap(!p.load(Ordering::Relaxed), Ordering::Relaxed));
    refresh_dom();
}

pub(crate) fn set_pending_resize() {
    PENDING_RESIZE.with(|p| p.store(true, Ordering::Relaxed));
    refresh_dom();
}

pub fn dom_update_receiver() -> DomUpdateReceiver {
    let rx = DOM_UPDATE_TX.with(|d| d.borrow().subscribe());
    DomUpdateReceiver(rx)
}

pub fn cleanup_removed_nodes(node: &DomNodeKey, remove: bool) {
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

pub fn clear_children(parent: DomNodeKey) {
    let children = with_nodes(|nodes| nodes[parent].children.clone());
    for child in children {
        unmount_child(child, true);
    }
}

pub fn unmount_child(child: DomNodeKey, cleanup: bool) {
    with_nodes_mut(|nodes| {
        nodes.unmount_child(child);
    });

    cleanup_removed_nodes(&child, cleanup);
    if !cleanup {
        with_nodes_mut(|nodes| nodes.set_unmounted(child, true));
    }
    refresh_dom();
}

pub fn print_dom() -> Paragraph<'static> {
    let lines =
        with_nodes(|nodes| print_dom_inner(nodes, nodes.root(0).as_dom_node().get_key(), ""));
    Paragraph::new(lines.clone()).wrap(Wrap { trim: false })
}

pub fn root() -> DomNodeRepr {
    with_nodes(|nodes| {
        let root = nodes.root(0).as_dom_node().get_key();
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

pub(crate) fn refresh_dom() {
    DOM_UPDATE_TX.with(|tx| {
        tx.borrow().send_modify(|_| {});
    });
}

pub fn mount<N>(node: N)
where
    N: AsDomNode + 'static,
{
    with_nodes_mut(|n| n.set_root(0, node));
}

pub fn unmount() {
    with_nodes_mut(|d| *d = NodeTree::new());
}

pub fn render_dom(frame: &mut Frame) {
    let buf = frame.buffer_mut();
    if PENDING_RESIZE.with(|p| p.swap(false, Ordering::Relaxed)) {
        with_nodes_mut(|nodes| nodes.set_window_size(buf.area));
    }

    if PRINT_DOM.with(|p| p.load(Ordering::Relaxed)) {
        print_dom().render_ref(buf.area, buf);
    } else {
        with_nodes_mut(|nodes| {
            nodes.recompute_full_layout(buf.area);
            nodes.clear_focusables();
        });

        let roots = with_nodes(|nodes| nodes.roots_asc());
        let window_area = *frame.buffer_mut().area();
        for root in roots {
            root.render(window_area, frame);
        }
    }
}

pub async fn render_terminal<B>(terminal: &mut NonblockingTerminal<B>) -> Result<(), io::Error>
where
    B: Backend + Send + Sync + 'static,
{
    draw(terminal, render_dom).await
}

async fn draw<B, F>(terminal: &mut NonblockingTerminal<B>, render_callback: F) -> io::Result<()>
where
    B: Backend + Send + Sync + 'static,
    F: FnOnce(&mut Frame),
{
    terminal.auto_resize().await;
    terminal.with_frame_mut(render_callback);
    terminal.draw().await;

    Ok(())
}

pub async fn render_single_frame<B>(terminal: &mut NonblockingTerminal<B>) -> Result<(), io::Error>
where
    B: Backend + Send + Sync + io::Write + 'static,
{
    render_terminal(terminal).await?;
    terminal.write(b"\n".into()).await;
    Ok(())
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
            nodes.set_focused(Some(node));
        }
    });
}

#[derive(thiserror::Error, Debug)]
#[error("node not found: {0:?}")]
pub struct NodeNotFound(pub NodeId);

pub fn focus_id(id: impl Into<NodeId>) {
    try_focus_id(id).expect("node not found")
}

pub fn try_focus_id(id: impl Into<NodeId>) -> Result<(), NodeNotFound> {
    let id = id.into();
    with_nodes_mut(|nodes| {
        let found_node = nodes.iter_nodes().find_map(|(key, node)| {
            if !node.inner.focusable {
                return None;
            }
            if let Some(current_id) = &node.inner.id {
                if &id == current_id {
                    return Some(key);
                }
            }
            None
        });
        if let Some(found_node) = found_node {
            nodes.set_focused(Some(found_node));
            Ok(())
        } else {
            Err(NodeNotFound(id))
        }
    })
}
