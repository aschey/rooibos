use std::cell::LazyCell;

use reactive_graph::signal::{ReadSignal, signal};
use reactive_graph::traits::{Get, SetValue, Update as _};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::with_nodes_mut;

use crate::derive_signal;
use crate::dom::NodeId;

thread_local! {
    static FOCUS_SIGNAL: LazyCell<ReadSignal<Option<NodeId>>> = LazyCell::new(|| {
        let (focus, set_focus) = signal::<Option<NodeId>>(None);
        let wrapper = NodeId::new_auto();

        with_nodes_mut(|nodes| {
            nodes.on_focus_change(move |id| {
                set_focus.update(|focused| match (&focused, id) {
                    (Some(f), Some(id)) => {
                        f.0.set_value(id);
                    }
                    (_, None) => {
                        *focused = None;
                    }
                    (None, Some(id)) => {
                        wrapper.0.set_value(id);
                        *focused = Some(wrapper);
                    }
                })
            })
        });
        focus
    });
}

pub fn use_focus() -> (NodeId, Signal<bool>) {
    let id = NodeId::new_auto();
    (id, use_focus_with_id_inner(id))
}

pub fn use_focus_with_id(id: impl Into<NodeId>) -> Signal<bool> {
    use_focus_with_id_inner(id.into())
}

pub fn use_focused_node() -> ReadSignal<Option<NodeId>> {
    FOCUS_SIGNAL.with(|f| **f)
}

fn use_focus_with_id_inner(id: NodeId) -> Signal<bool> {
    let focused_node = use_focused_node();
    derive_signal!(focused_node.get().map(|node| node == id).unwrap_or(false))
}
