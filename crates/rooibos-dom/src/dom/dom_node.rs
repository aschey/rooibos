use core::fmt::Debug;
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::fmt::{self, Display};
use std::rc::Rc;

use derivative::Derivative;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::widgets::Block;
use ratatui::Frame;
use slotmap::{new_key_type, SlotMap};
use tachys::view::Render;

use super::document_fragment::DocumentFragment;
use super::dom_state::DomState;
use super::{with_nodes, with_nodes_mut, with_state_mut};
use crate::{next_node_id, DomWidgetNode, EventHandlers, RooibosDom};

#[derive(Clone, PartialEq, Eq, Debug)]
enum NodeIdInner {
    Auto(u32),
    Manual(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeId(NodeIdInner);

impl NodeId {
    pub fn new_auto() -> Self {
        Self(NodeIdInner::Auto(next_node_id()))
    }

    pub fn new(id: impl Into<String>) -> Self {
        Self(NodeIdInner::Manual(id.into()))
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            NodeIdInner::Auto(val) => std::fmt::Display::fmt(&val, f),
            NodeIdInner::Manual(val) => std::fmt::Display::fmt(&val, f),
        }
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

#[derive(PartialEq, Eq, Clone, Default)]
pub(crate) struct LayoutProps {
    pub(crate) direction: Direction,
    pub(crate) flex: Flex,
    pub(crate) margin: u16,
    pub(crate) spacing: u16,
    pub(crate) block: Option<Block<'static>>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum NodeType {
    Layout(Rc<RefCell<LayoutProps>>),
    Overlay,
    Widget(DomWidgetNode),
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
                    block,
                } = layout_props.borrow().clone();
                NodeTypeStructure {
                    name: "Layout",
                    attrs: Some(format!(
                        "direction={direction}, flex={flex}, margin={margin}, spacing={spacing}, \
                         block={block:?}"
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
                    block,
                    spacing,
                } = layout_props.borrow().clone();
                write!(
                    f,
                    "Layout(direction={direction}, flex={flex}, margin={margin}, \
                     spacing={spacing}, block={block:?})"
                )
            }
            NodeType::Overlay => write!(f, "Overlay"),
            NodeType::Widget(widget) => write!(f, "Widget({widget:?})"),
        }
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct DomNodeInner {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) constraint: Rc<RefCell<Constraint>>,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    // pub(crate) before_pending: Vec<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) focusable: Rc<RefCell<bool>>,
    #[derivative(Debug = "ignore")]
    pub(crate) event_handlers: EventHandlers,
    pub(crate) rect: Rc<RefCell<Rect>>,
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
        *self.rect.borrow_mut() = rect;
        if *self.focusable.borrow() {
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
                    mut margin,
                    spacing,
                    block,
                } = layout_props.borrow().clone();
                if let Some(block) = block {
                    // Need margin to prevent block from rendering over the content
                    if margin < 1 {
                        margin = 1;
                    }
                    frame.render_widget_ref(block, rect);
                };
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
                *self.rect.borrow_mut() = chunks[0];
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
            // before_pending: vec![],
            focusable: Rc::new(RefCell::new(fragment.focusable)),
            id: fragment.id,
            event_handlers: fragment.event_handlers,
            data: vec![],
            rect: Rc::new(RefCell::new(Rect::default())),
        };
        let key = with_nodes_mut(|mut n| n.insert(inner));
        Self { key }
    }

    pub(crate) fn replace_fragment(&self, fragment: DocumentFragment) {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: Rc::new(RefCell::new(fragment.constraint)),
            children: vec![],
            parent: self.get_parent_key(),
            // before_pending: vec![],
            focusable: Rc::new(RefCell::new(fragment.focusable)),
            id: fragment.id,
            event_handlers: fragment.event_handlers,
            data: vec![],
            rect: Rc::new(RefCell::new(Rect::default())),
        };
        with_nodes_mut(|mut n| n[self.key] = inner);
    }

    pub(crate) fn add_data(&self, data: impl Any) {
        with_nodes_mut(|mut n| n[self.key].data.push(Rc::new(data)));
    }

    // pub(crate) fn set_name(&self, name: impl Into<String>) {
    //     DOM_NODES.with(|n| n.borrow_mut()[self.key].name = name.into());
    // }

    pub(crate) fn set_constraint(&self, constraint: Rc<RefCell<Constraint>>) {
        with_nodes_mut(|mut n| n[self.key].constraint = constraint);
    }

    pub(crate) fn set_focusable(&self, focusable: Rc<RefCell<bool>>) {
        with_nodes_mut(|mut n| n[self.key].focusable = focusable);
    }

    pub(crate) fn update_event_handlers<F>(&self, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        with_nodes_mut(|mut n| {
            n[self.key].event_handlers = update(n[self.key].event_handlers.clone())
        });
    }

    pub(crate) fn set_id(&self, id: impl Into<NodeId>) {
        with_nodes_mut(|mut n| {
            n[self.key].id = Some(id.into());
            *n[self.key].focusable.borrow_mut() = true;
        });
    }

    pub(crate) fn layout_props(&self) -> Rc<RefCell<LayoutProps>> {
        with_nodes(|n| {
            if let NodeType::Layout(layout_props) = &n[self.key].node_type {
                layout_props.clone()
            } else {
                Rc::new(RefCell::new(LayoutProps::default()))
            }
        })
    }

    pub(crate) fn key(&self) -> DomNodeKey {
        self.key
    }

    pub(crate) fn get_parent_key(&self) -> Option<DomNodeKey> {
        with_nodes(|n| n[self.key].parent)
    }

    pub(crate) fn get_parent(&self) -> Option<DomNode> {
        with_nodes(|n| n[self.key].parent.map(|p| DomNode { key: p }))
    }

    pub(crate) fn append_child(&self, node: &DomNode) {
        with_nodes_mut(|mut d| {
            d[node.key].parent = Some(self.key);
            d[self.key].children.push(node.key);

            // let pending: Vec<_> = d[node.key].before_pending.drain(..).collect();
            // for p in pending {
            //     let self_index = d[self.key]
            //         .children
            //         .iter()
            //         .position(|c| c == &node.key)
            //         .unwrap();
            //     d[self.key].children.insert(self_index, p);
            //     d[p].parent = Some(self.key);
            // }
        });
    }

    // pub(crate) fn before(&self, node: &DomNode) {
    //     DOM_NODES.with(|d| {
    //         let mut d = d.borrow_mut();

    //         if let Some(parent_id) = d[self.key].parent {
    //             let parent = d.get_mut(parent_id).unwrap();
    //             let self_index = parent.children.iter().position(|c| c == &self.key).unwrap();
    //             parent.children.insert(self_index, node.key);
    //             d[node.key].parent = Some(parent_id);
    //         } else {
    //             d[self.key].before_pending.push(node.key);
    //         }
    //     });
    // }

    pub(crate) fn render(&self, frame: &mut Frame, rect: Rect) {
        with_nodes(|d| {
            with_state_mut(|state| {
                state.clear_focusables();
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

    fn build(self) -> Self::State {
        self
    }

    fn rebuild(self, _state: &mut Self::State) {}
}
