use std::cell::LazyCell;

use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::{Get, Set as _};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom2::{with_nodes_mut, NodeId};

use crate::derive_signal;

thread_local! {
    static FOCUS_SIGNAL: LazyCell<ReadSignal<Option<NodeId>>> = LazyCell::new(|| {
        let (focus, set_focus) = signal(None);
        with_nodes_mut(|nodes| nodes.on_focus_change(move |id| set_focus.set(id)));
        focus
    });
}

pub fn use_focus() -> (NodeId, impl Get<Value = bool> + Copy) {
    let id = NodeId::new_auto();
    use_focus_with_id_inner(id)
}

pub fn use_focus_with_id(id: impl Into<String>) -> (NodeId, impl Get<Value = bool> + Copy) {
    let id = NodeId::new(id);
    use_focus_with_id_inner(id)
}

pub fn use_focused_node() -> ReadSignal<Option<NodeId>> {
    FOCUS_SIGNAL.with(|f| **f)
}

fn use_focus_with_id_inner(id: NodeId) -> (NodeId, Signal<bool>) {
    let focused_node = use_focused_node();
    let focused = {
        let id = id.clone();
        derive_signal!(focused_node.get().map(|node| node == id).unwrap_or(false))
    };

    (id, focused)
}
