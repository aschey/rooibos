use core::fmt::Debug;
use std::cell::{Ref, RefMut};
use std::fmt;
use std::sync::atomic::Ordering;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::Frame;
use slotmap::{new_key_type, SlotMap};

use super::document_fragment::DocumentFragment;
use super::dom_state::DomState;
use super::dom_widget::DomWidget;
use super::view::{IntoView, View};
use super::{DOM_NODES, DOM_STATE};
use crate::{next_node_id, Mountable};

#[derive(Clone, PartialEq, Eq)]
enum NodeIdInner {
    Auto(u32),
    Manual(String),
}

#[derive(Clone, PartialEq, Eq)]
pub struct NodeId(NodeIdInner);

impl NodeId {
    pub fn new_auto() -> Self {
        Self(NodeIdInner::Auto(next_node_id()))
    }

    pub fn new(id: impl Into<String>) -> Self {
        Self(NodeIdInner::Manual(id.into()))
    }
}

impl From<String> for NodeId {
    fn from(val: String) -> Self {
        NodeId(NodeIdInner::Manual(val))
    }
}

impl From<&str> for NodeId {
    fn from(val: &str) -> Self {
        NodeId(NodeIdInner::Manual(val.to_string()))
    }
}

new_key_type! {pub(crate) struct DomNodeKey; }

pub(crate) struct NodeTypeStructure {
    pub(crate) name: &'static str,
    pub(crate) attrs: Option<String>,
    pub(crate) children: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum NodeType {
    Layout {
        direction: Direction,
        flex: Flex,
        margin: u16,
        spacing: u16,
    },
    Transparent,
    Overlay,
    Widget(DomWidget),
}

impl NodeType {
    pub(crate) fn structure(&self) -> NodeTypeStructure {
        match self {
            NodeType::Layout {
                direction,
                flex,
                margin,
                spacing,
            } => NodeTypeStructure {
                name: "Layout",
                attrs: Some(format!(
                    "direction={direction}, flex={flex}, margin={margin}, spacing={spacing}"
                )),
                children: None,
            },

            NodeType::Transparent => NodeTypeStructure {
                name: "Transparent",
                attrs: None,
                children: None,
            },
            NodeType::Overlay => NodeTypeStructure {
                name: "Overlay",
                attrs: None,
                children: None,
            },
            NodeType::Widget(widget) => NodeTypeStructure {
                name: "Widget",
                attrs: None,
                children: Some(format!("{widget:?}")),
            },
        }
    }
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::Layout {
                direction,
                flex,
                margin,
                spacing,
            } => write!(
                f,
                "Layout(direction={direction}, flex={flex}, margin={margin}, spacing={spacing})"
            ),

            NodeType::Transparent => write!(f, "Transparent"),
            // NodeType::Root => write!(f, "Root"),
            NodeType::Overlay => write!(f, "Overlay"),
            NodeType::Widget(widget) => write!(f, "Widget({widget:?})"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct DomNodeInner {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) constraint: Constraint,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) before_pending: Vec<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) focusable: bool,
}

impl DomNodeInner {
    pub(crate) fn resolve_children(
        &self,
        dom_nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    ) -> Vec<(DomNodeKey, DomNodeInner)> {
        let children: Vec<_> = self
            .children
            .iter()
            .flat_map(|c| {
                let child = &dom_nodes[*c];
                if child.node_type == NodeType::Transparent {
                    return child.resolve_children(dom_nodes);
                }
                vec![(*c, child.to_owned())]
            })
            .collect();
        children
    }

    fn render(
        &self,
        frame: &mut Frame,
        rect: Rect,
        key: DomNodeKey,
        dom_nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
        dom_state: &mut RefMut<'_, DomState>,
    ) {
        if self.focusable {
            dom_state.add_focusable(key);
        }
        let children: Vec<_> = self.resolve_children(dom_nodes);

        let constraints = children.iter().map(|(_, c)| c.constraint);

        match &self.node_type {
            NodeType::Layout {
                direction,
                margin,
                flex,
                spacing,
            } => {
                let layout = Layout::default()
                    .direction(*direction)
                    .flex(*flex)
                    .margin(*margin)
                    .spacing(*spacing)
                    .constraints(constraints);

                let chunks = layout.split(rect);
                children
                    .iter()
                    .zip(chunks.iter())
                    .for_each(|((key, child), chunk)| {
                        child.render(frame, *chunk, *key, dom_nodes, dom_state);
                    });
            }

            NodeType::Overlay | NodeType::Transparent => {
                children.iter().for_each(|(key, child)| {
                    child.render(frame, rect, *key, dom_nodes, dom_state);
                });
            }
            NodeType::Widget(widget) => {
                widget.render(frame, rect);
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DomNode {
    key: DomNodeKey,
}

impl DomNode {
    pub(crate) fn from_fragment(fragment: DocumentFragment) -> Self {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: fragment.constraint,
            children: vec![],
            parent: None,
            before_pending: vec![],
            focusable: fragment.id.is_some(),
            id: fragment.id,
        };
        let key = DOM_NODES.with(|n| n.borrow_mut().insert(inner));
        Self { key }
    }

    pub(crate) fn transparent(name: impl Into<String>) -> Self {
        Self::from_fragment(DocumentFragment::transparent(name))
    }

    pub(crate) fn set_name(&self, name: impl Into<String>) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].name = name.into());
    }

    pub(crate) fn set_constraint(&self, constraint: Constraint) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].constraint = constraint);
    }

    pub(crate) fn set_id(&self, id: impl Into<NodeId>) {
        DOM_NODES.with(|n| {
            let mut n = n.borrow_mut();
            n[self.key].id = Some(id.into());
            n[self.key].focusable = true;
        });
    }

    pub(crate) fn set_margin(&self, new_margin: u16) {
        DOM_NODES.with(|n| {
            if let NodeType::Layout { margin, .. } = &mut n.borrow_mut()[self.key].node_type {
                *margin = new_margin;
            }
        });
    }

    pub(crate) fn key(&self) -> DomNodeKey {
        self.key
    }

    pub(crate) fn append_child(&self, node: &DomNode) {
        DOM_NODES.with(|d| {
            let mut d = d.borrow_mut();

            d[node.key].parent = Some(self.key);
            d[self.key].children.push(node.key);

            let pending: Vec<_> = d[node.key].before_pending.drain(..).collect();
            for p in pending {
                let self_index = d[self.key]
                    .children
                    .iter()
                    .position(|c| c == &node.key)
                    .unwrap();
                d[self.key].children.insert(self_index, p);
                d[p].parent = Some(self.key);
            }
        });
    }

    pub(crate) fn before(&self, node: &DomNode) {
        DOM_NODES.with(|d| {
            let mut d = d.borrow_mut();

            if let Some(parent_id) = d[self.key].parent {
                let parent = d.get_mut(parent_id).unwrap();
                let self_index = parent.children.iter().position(|c| c == &self.key).unwrap();
                parent.children.insert(self_index, node.key);
                d[node.key].parent = Some(parent_id);
            } else {
                d[self.key].before_pending.push(node.key);
            }
        });
    }

    pub(crate) fn render(&self, frame: &mut Frame, rect: Rect) {
        DOM_NODES.with(|d| {
            let d = d.borrow();
            DOM_STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.clear_focused();
                d[self.key].render(frame, rect, self.key, &d, &mut state);
            });
        });
    }
}

impl IntoView for DomNode {
    fn into_view(self) -> View {
        View::DomNode(self)
    }
}

impl Mountable for DomNode {
    fn get_mountable_node(&self) -> DomNode {
        self.clone()
    }
}
