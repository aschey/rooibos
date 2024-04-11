use core::fmt::Debug;
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::Frame;
use slotmap::{new_key_type, SlotMap};
use tachys::view::Render;

use super::document_fragment::DocumentFragment;
use super::dom_state::DomState;
use super::dom_widget::DomWidget;
// use super::view::{IntoView, View};
use super::{with_state_mut, DOM_NODES};
use crate::{next_node_id, RooibosDom};

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

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub(crate) struct LayoutProps {
    pub(crate) direction: Direction,
    pub(crate) flex: Flex,
    pub(crate) margin: u16,
    pub(crate) spacing: u16,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum NodeType {
    Layout(Rc<RefCell<LayoutProps>>),
    Overlay,
    Widget(DomWidget),
}

impl NodeType {
    pub(crate) fn structure(&self) -> NodeTypeStructure {
        match self {
            NodeType::Layout(layout_props) => {
                let LayoutProps {
                    direction,
                    flex,
                    margin,
                    spacing,
                } = *layout_props.borrow();
                NodeTypeStructure {
                    name: "Layout",
                    attrs: Some(format!(
                        "direction={direction}, flex={flex}, margin={margin}, spacing={spacing}"
                    )),
                    children: None,
                }
            }
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
            NodeType::Layout(layout_props) => {
                let LayoutProps {
                    direction,
                    flex,
                    margin,
                    spacing,
                } = *layout_props.borrow();
                write!(
                    f,
                    "Layout(direction={direction}, flex={flex}, margin={margin}, \
                     spacing={spacing})"
                )
            }
            NodeType::Overlay => write!(f, "Overlay"),
            NodeType::Widget(widget) => write!(f, "Widget({widget:?})"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct DomNodeInner {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) constraint: Rc<RefCell<Constraint>>,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) before_pending: Vec<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) focusable: bool,
    data: Vec<Rc<dyn Any>>,
}

impl DomNodeInner {
    fn render(
        &self,
        frame: &mut Frame,
        rect: Rect,
        key: DomNodeKey,
        dom_nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
        dom_state: &mut DomState,
        parent_layout: Layout,
    ) {
        if self.focusable {
            dom_state.add_focusable(key);
        }

        let constraints = self
            .children
            .iter()
            .map(|key| *dom_nodes[*key].constraint.borrow());

        match &self.node_type {
            NodeType::Layout(layout_props) => {
                let LayoutProps {
                    direction,
                    flex,
                    margin,
                    spacing,
                } = *layout_props.borrow();

                let layout = Layout::default()
                    .direction(direction)
                    .flex(flex)
                    .margin(margin)
                    .spacing(spacing)
                    .constraints(constraints);

                let chunks = layout.split(rect);
                self.children
                    .iter()
                    .zip(chunks.iter())
                    .for_each(|(key, chunk)| {
                        dom_nodes[*key].render(
                            frame,
                            *chunk,
                            *key,
                            dom_nodes,
                            dom_state,
                            Layout::default(),
                        );
                    });
            }

            NodeType::Overlay => {
                let parent_layout = parent_layout.constraints([*self.constraint.borrow()]);
                let chunks = parent_layout.split(rect);
                self.children.iter().for_each(|key| {
                    dom_nodes[*key].render(
                        frame,
                        chunks[0],
                        *key,
                        dom_nodes,
                        dom_state,
                        parent_layout.clone(),
                    );
                });
            }
            NodeType::Widget(widget) => {
                let parent_layout = parent_layout.constraints([*self.constraint.borrow()]);
                let chunks = parent_layout.split(rect);
                widget.render(frame, chunks[0]);
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DomNode {
    key: DomNodeKey,
}

impl DomNode {
    // pub(crate) fn from_key(key: DomNodeKey) -> Self {
    //     Self { key }
    // }

    pub(crate) fn from_fragment(fragment: DocumentFragment) -> Self {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: Rc::new(RefCell::new(fragment.constraint)),
            children: vec![],
            parent: None,
            before_pending: vec![],
            focusable: fragment.id.is_some(),
            id: fragment.id,
            data: vec![],
        };
        let key = DOM_NODES.with(|n| n.borrow_mut().insert(inner));
        Self { key }
    }

    pub(crate) fn replace_fragment(&self, fragment: DocumentFragment) {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: Rc::new(RefCell::new(fragment.constraint)),
            children: vec![],
            parent: None,
            before_pending: vec![],
            focusable: fragment.id.is_some(),
            id: fragment.id,
            data: vec![],
        };
        DOM_NODES.with(|n| n.borrow_mut()[self.key] = inner);
    }

    pub(crate) fn add_data(&self, data: impl Any) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].data.push(Rc::new(data)));
    }

    pub(crate) fn set_name(&self, name: impl Into<String>) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].name = name.into());
    }

    pub(crate) fn set_constraint(&self, constraint: Rc<RefCell<Constraint>>) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].constraint = constraint);
    }

    pub(crate) fn set_id(&self, id: impl Into<NodeId>) {
        DOM_NODES.with(|n| {
            let mut n = n.borrow_mut();
            n[self.key].id = Some(id.into());
            n[self.key].focusable = true;
        });
    }

    pub(crate) fn layout_props(&self) -> Rc<RefCell<LayoutProps>> {
        DOM_NODES.with(|n| {
            let mut n = n.borrow_mut();
            if let NodeType::Layout(layout_props) = &mut n[self.key].node_type {
                layout_props.clone()
            } else {
                Rc::new(RefCell::new(LayoutProps::default()))
            }
        })
    }

    pub(crate) fn key(&self) -> DomNodeKey {
        self.key
    }

    pub(crate) fn get_parent(&self) -> Option<DomNode> {
        DOM_NODES.with(|n| {
            let n = n.borrow();
            n[self.key].parent.map(|p| DomNode { key: p })
        })
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
            with_state_mut(|state| {
                state.clear_focused();
                let constraint = *d[self.key].constraint.borrow();

                d[self.key].render(
                    frame,
                    rect,
                    self.key,
                    &d,
                    state,
                    Layout::default().constraints([constraint]),
                );
            });
        });
    }
}

impl Render<RooibosDom> for DomNode {
    type State = DomNode;

    type FallibleState = ();

    type AsyncOutput = ();

    fn build(self) -> Self::State {
        self
    }

    fn rebuild(self, state: &mut Self::State) {}

    fn try_build(self) -> any_error::Result<Self::FallibleState> {
        todo!()
    }

    fn try_rebuild(self, state: &mut Self::FallibleState) -> any_error::Result<()> {
        todo!()
    }

    async fn resolve(self) -> Self::AsyncOutput {
        todo!()
    }
}
