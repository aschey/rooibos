use ratatui::layout::Rect;
use reactive_graph::signal::{signal, ReadSignal, WriteSignal};
use reactive_graph::traits::Set;
use slotmap::SlotMap;

use super::dom_node::{DomNodeInner, DomNodeKey, NodeId};
use crate::EventData;

pub(crate) struct DomState {
    window_size: ReadSignal<Rect>,
    set_window_size: WriteSignal<Rect>,
    focused: ReadSignal<Option<NodeId>>,
    set_focused: WriteSignal<Option<NodeId>>,
    focused_key: Option<DomNodeKey>,
    hovered_key: Option<DomNodeKey>,
    focusable_nodes: Vec<DomNodeKey>,
}

impl Default for DomState {
    fn default() -> Self {
        let (focused, set_focused) = signal(None);
        let (window_size, set_window_size) = signal(Rect::default());
        Self {
            window_size,
            set_window_size,
            focused,
            set_focused,
            focused_key: None,
            hovered_key: None,
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

    pub(crate) fn hovered_key(&self) -> Option<DomNodeKey> {
        self.hovered_key
    }

    pub(crate) fn window_size(&self) -> ReadSignal<Rect> {
        self.window_size
    }

    pub(crate) fn set_window_size(&self, size: Rect) {
        self.set_window_size.set(size);
    }

    pub(crate) fn focusable_nodes(&self) -> &Vec<DomNodeKey> {
        &self.focusable_nodes
    }

    pub(crate) fn set_focused(
        &mut self,
        node_key: DomNodeKey,
        nodes: &mut SlotMap<DomNodeKey, DomNodeInner>,
    ) {
        if let Some(focused_key) = self.focused_key {
            let node = &mut nodes[focused_key];
            if let Some(on_blur) = &mut node.event_handlers.on_blur {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                on_blur.borrow_mut()(EventData {
                    rect: *node.rect.borrow(),
                });
            }
        }
        self.focused_key = Some(node_key);
        let node = &mut nodes[node_key];
        self.set_focused.set(node.id.to_owned());
        if let Some(on_focused) = &mut node.event_handlers.on_focus {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            on_focused.borrow_mut()(EventData {
                rect: *node.rect.borrow(),
            });
        }
    }

    pub(crate) fn set_hovered(
        &mut self,
        node_key: DomNodeKey,
        nodes: &mut SlotMap<DomNodeKey, DomNodeInner>,
    ) {
        if let Some(hovered_key) = self.hovered_key {
            let node = &mut nodes[hovered_key];
            if let Some(on_mouse_leave) = &mut node.event_handlers.on_mouse_leave {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                on_mouse_leave.borrow_mut()(EventData {
                    rect: *node.rect.borrow(),
                });
            }
        }
        self.hovered_key = Some(node_key);
        let node = &mut nodes[node_key];
        if let Some(on_mouse_enter) = &mut node.event_handlers.on_mouse_enter {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            on_mouse_enter.borrow_mut()(EventData {
                rect: *node.rect.borrow(),
            });
        }
    }

    pub(crate) fn remove_hovered(&mut self, nodes: &mut SlotMap<DomNodeKey, DomNodeInner>) {
        if let Some(node_key) = self.hovered_key {
            let node = &mut nodes[node_key];
            if let Some(on_mouse_leave) = &mut node.event_handlers.on_mouse_leave {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                on_mouse_leave.borrow_mut()(EventData {
                    rect: *node.rect.borrow(),
                });
            }
        }
        self.hovered_key = None;
    }

    pub(crate) fn cleanup_before_remove(&mut self, key: &DomNodeKey) {
        if self.focused_key == Some(*key) {
            self.focused_key = None;
            self.set_focused.set(None);
        }
        if self.hovered_key == Some(*key) {
            self.hovered_key = None;
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
