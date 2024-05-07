use std::cell::RefMut;

use reactive_graph::signal::{signal, ReadSignal, WriteSignal};
use reactive_graph::traits::Set;
use slotmap::SlotMap;

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

    pub(crate) fn set_focused(
        &mut self,
        node_key: DomNodeKey,
        nodes: &mut RefMut<SlotMap<DomNodeKey, DomNodeInner>>,
    ) {
        if let Some(focused_key) = self.focused_key {
            if let Some(on_blur) = &mut nodes[focused_key].event_handlers.on_blur {
                on_blur.borrow_mut()();
            }
        }
        self.focused_key = Some(node_key);
        let node = &mut nodes[node_key];
        self.set_focused.set(node.id.to_owned());
        if let Some(on_focused) = &mut node.event_handlers.on_focus {
            on_focused.borrow_mut()();
        }
    }

    pub(crate) fn remove_focusable(&mut self, key: &DomNodeKey) {
        if self.focused_key == Some(*key) {
            self.focused_key = None;
        }
        if let Some(pos) = self.focusable_nodes.iter().position(|n| n == key) {
            self.focusable_nodes.remove(pos);
        }
    }

    pub(crate) fn clear_focusables(&mut self) {
        self.focusable_nodes.clear();
    }

    pub(crate) fn add_focusable(&mut self, key: DomNodeKey) {
        self.focusable_nodes.push(key);
    }
}
