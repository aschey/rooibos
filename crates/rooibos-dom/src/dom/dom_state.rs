use reactive_graph::signal::{signal, ReadSignal, WriteSignal};
use reactive_graph::traits::Set;

use super::dom_node::{DomNodeInner, DomNodeKey, NodeId};

pub(crate) struct DomState {
    focused: ReadSignal<Option<NodeId>>,
    set_focused: WriteSignal<Option<NodeId>>,
    focused_key: Option<DomNodeKey>,
    focusable_nodes: Vec<DomNodeKey>,
}

impl Default for DomState {
    fn default() -> Self {
        let (focused, set_focused) = signal(None);
        Self {
            focused,
            set_focused,
            focused_key: None,
            focusable_nodes: vec![],
        }
    }
}

impl DomState {
    pub(crate) fn focused(&self) -> ReadSignal<Option<NodeId>> {
        self.focused
    }

    pub(crate) fn focused_key(&self) -> Option<DomNodeKey> {
        self.focused_key
    }

    pub(crate) fn focusable_nodes(&self) -> &Vec<DomNodeKey> {
        &self.focusable_nodes
    }

    pub(crate) fn set_focused(&mut self, node_key: Option<DomNodeKey>, node: &DomNodeInner) {
        self.focused_key = node_key;
        self.set_focused.set(node.id.to_owned());
    }

    pub(crate) fn clear_focused(&mut self) {
        self.focusable_nodes.clear();
    }

    pub(crate) fn add_focusable(&mut self, key: DomNodeKey) {
        self.focusable_nodes.push(key);
    }
}
