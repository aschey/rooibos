use reactive_graph::computed::Memo;
use reactive_graph::traits::Get;

use crate::{focused_node, NodeId};

pub fn use_focus() -> (NodeId, impl Get<Value = bool> + Copy) {
    let id = NodeId::new_auto();
    use_focus_with_id_inner(id)
}

pub fn use_focus_with_id(id: impl Into<String>) -> (NodeId, impl Get<Value = bool> + Copy) {
    let id = NodeId::new(id);
    use_focus_with_id_inner(id)
}

fn use_focus_with_id_inner(id: NodeId) -> (NodeId, impl Get<Value = bool> + Copy) {
    let focused_node = focused_node();
    let focused = Memo::new({
        let id = id.clone();
        move |_| focused_node.get().map(|node| node == id).unwrap_or(false)
    });

    (id, focused)
}
