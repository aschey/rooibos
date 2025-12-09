use std::cell::LazyCell;

use reactive_graph::owner::{ArcStoredValue, StoredValue};
use reactive_graph::signal::{ArcReadSignal, arc_signal};
use reactive_graph::traits::{Get, GetValue, SetValue, Update as _};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::with_nodes_mut;

use crate::IntoSignal;
use crate::dom::NodeId;

thread_local! {
    static FOCUS_SIGNAL: LazyCell<ArcReadSignal<Option<rooibos_dom::NodeId>>> = LazyCell::new(|| {
        let (focus, set_focus) = arc_signal::<Option<rooibos_dom::NodeId>>(None);
        let  wrapper = ArcStoredValue::new(rooibos_dom::NodeId::new_auto());

        with_nodes_mut(|nodes| {
            nodes.on_focus_change(move |id| {
                set_focus.update(|focused| match (focused, id) {
                    (Some(f), Some(id)) => {
                        *f = id;
                    }
                    (f, None) => {
                        *f = None;
                    }
                    (f, Some(id)) => {
                        wrapper.set_value(id);
                        *f = Some(wrapper.get_value());
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

pub fn use_focused_node() -> Signal<Option<NodeId>> {
    FOCUS_SIGNAL.with(|f| {
        let f = (**f).clone();
        (move || f.get().map(|f| NodeId(StoredValue::new(f)))).signal()
    })
}

fn use_focus_with_id_inner(id: NodeId) -> Signal<bool> {
    let focused_node = use_focused_node();
    (move || focused_node.get().map(|node| node == id).unwrap_or(false)).signal()
}
