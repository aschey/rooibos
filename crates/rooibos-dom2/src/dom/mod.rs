use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub use dom_node::*;
pub use dom_widget::*;
pub use node_tree::*;
use ratatui::buffer::Buffer;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use tokio::sync::watch;
use tracing::error;

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
    with_nodes_mut(|nodes| nodes.set_unmounted(child, true));
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

pub fn refresh_dom() {
    let _ = DOM_UPDATE_TX.with(|tx| {
        tx.borrow()
            .send(())
            .inspect_err(|e| error!("error sending DOM update: {e:?}"))
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

pub fn render_dom(buf: &mut Buffer) {
    if PENDING_RESIZE.with(|p| p.swap(false, Ordering::Relaxed)) {
        with_nodes_mut(|nodes| nodes.set_window_size(buf.area));
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
            nodes.set_focused(Some(node));
        }
    });
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
            nodes.set_focused(Some(found_node));
        }
    });
}
