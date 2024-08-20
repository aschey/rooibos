use core::fmt::Debug;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::{self};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Clear, Widget, WidgetRef};
use reactive_graph::effect::RenderEffect;
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};
use terminput::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::dom_state::{self, DomState};
use super::node_tree::{DomNodeKey, NodeTree};
use super::{unmount_child, with_nodes, with_nodes_mut, with_state_mut, AsDomNode, DOM_NODES};
use crate::{next_node_id, send_event, DomWidgetNode, EventHandlers, Role, RooibosDom};

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

impl fmt::Display for NodeId {
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

pub(crate) struct NodeTypeStructure {
    pub(crate) name: &'static str,
    pub(crate) attrs: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub enum NodeType {
    Layout,
    Widget(DomWidgetNode),
    #[default]
    Placeholder,
}

impl NodeType {
    pub(crate) fn structure(&self) -> NodeTypeStructure {
        match self {
            NodeType::Layout => NodeTypeStructure {
                name: "Layout",
                attrs: None,
            },
            NodeType::Widget(_) => NodeTypeStructure {
                name: "Widget",
                attrs: None,
            },
            NodeType::Placeholder => NodeTypeStructure {
                name: "Placeholder",
                attrs: None,
            },
        }
    }
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::Layout => {
                write!(f, "Layout")
            }
            NodeType::Widget(widget) => write!(f, "Widget({widget:?})"),
            NodeType::Placeholder => write!(f, "Placeholder"),
        }
    }
}

impl Render<RooibosDom> for NodeType {
    type State = Option<RenderEffect<()>>;

    fn build(self) -> Self::State {
        match self {
            NodeType::Layout => None,
            NodeType::Widget(node) => Some(node.build()),
            NodeType::Placeholder => None,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match self {
            NodeType::Layout => {}
            NodeType::Widget(node) => {
                if let Some(s) = state.as_mut() {
                    node.rebuild(s)
                }
            }
            NodeType::Placeholder => {}
        }
    }
}

pub trait AnyMountable: Mountable<RooibosDom> + Any {
    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> AnyMountable for T
where
    T: Mountable<RooibosDom> + Any,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Mountable<RooibosDom> for Box<dyn AnyMountable> {
    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        (**self).mount(parent, marker)
    }

    fn unmount(&mut self) {
        (**self).unmount()
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        (**self).insert_before_this(child)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeTypeRepr {
    Layout { block: Option<Block<'static>> },
    Overlay,
    Absolute,
    Widget,
    Placeholder,
}

#[derive(Clone)]
pub struct DomNodeRepr {
    key: DomNodeKey,
    rect: Rect,
    node_type: NodeTypeRepr,
}

impl DomNodeRepr {
    pub(crate) fn from_node(key: DomNodeKey, node: &DomNodeInner) -> Self {
        Self {
            key,
            rect: *node.rect.borrow(),
            node_type: match &node.node_type {
                NodeType::Layout => NodeTypeRepr::Layout {
                    block: node.block.clone(),
                },
                NodeType::Widget(_) => NodeTypeRepr::Widget,
                NodeType::Placeholder => NodeTypeRepr::Placeholder,
            },
        }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn node_type(&self) -> NodeTypeRepr {
        self.node_type.clone()
    }

    pub fn find<F>(&self, f: F) -> Option<DomNodeRepr>
    where
        F: Fn(&DomNodeRepr) -> bool + Clone,
    {
        self.key
            .traverse(
                |key, node| {
                    let repr = DomNodeRepr::from_node(key, node);
                    if f(&repr) { Some(repr) } else { None }
                },
                true,
            )
            .first()
            .cloned()
    }

    pub fn get<F>(&self, f: F) -> DomNodeRepr
    where
        F: Fn(&DomNodeRepr) -> bool + Clone,
    {
        self.find(f).unwrap()
    }

    pub fn find_all<F>(&self, f: F) -> Vec<DomNodeRepr>
    where
        F: Fn(&DomNodeRepr) -> bool + Clone,
    {
        self.key.traverse(
            |key, node| {
                let repr = DomNodeRepr::from_node(key, node);
                if f(&repr) { Some(repr) } else { None }
            },
            false,
        )
    }

    pub fn find_by_id(&self, id: impl Into<NodeId>) -> Option<DomNodeRepr> {
        let id = id.into();
        let nodes = self.key.traverse(
            |key, node| {
                if node.id.as_ref() == Some(&id) {
                    Some(DomNodeRepr::from_node(key, node))
                } else {
                    None
                }
            },
            true,
        );
        nodes.first().cloned()
    }

    pub fn get_by_id(&self, id: impl Into<NodeId>) -> DomNodeRepr {
        self.find_by_id(id).unwrap()
    }

    pub fn find_all_by_role(&self, role: Role) -> Vec<DomNodeRepr> {
        self.key.traverse(
            |key, node| {
                if let NodeType::Widget(widget_node) = &node.node_type {
                    if widget_node.role == Some(role) {
                        return Some(DomNodeRepr::from_node(key, node));
                    }
                }
                None
            },
            false,
        )
    }

    pub fn find_all_by_class(&self, class: impl Into<String>) -> Vec<DomNodeRepr> {
        let class = class.into();
        self.key.traverse(
            |key, node| {
                if node.class.as_ref() == Some(&class) {
                    return Some(DomNodeRepr::from_node(key, node));
                }

                None
            },
            false,
        )
    }

    pub fn children_count(&self) -> usize {
        with_nodes(|nodes| nodes[self.key].children.len())
    }

    pub fn click(&self) {
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: self.rect.x,
            row: self.rect.y,
            modifiers: KeyModifiers::empty(),
        });
        send_event(event);
    }

    pub fn focus(&self) {
        let found_node = with_nodes(|nodes| {
            nodes
                .iter_nodes()
                .find_map(|(key, _)| if key == self.key { Some(key) } else { None })
        })
        .unwrap();
        dom_state::set_focused(found_node);
    }
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct DomNodeInner {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) original_display: taffy::Display,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) class: Option<String>,
    pub(crate) focusable: bool,
    #[derivative(Debug = "ignore")]
    pub(crate) event_handlers: EventHandlers,
    pub(crate) rect: Rc<RefCell<Rect>>,
    pub(crate) z_index: Option<i32>,
    pub(crate) block: Option<Block<'static>>,
    pub(crate) clear: bool,
    pub(crate) unmounted: Arc<AtomicBool>,
}

impl Render<RooibosDom> for DomNodeInner {
    type State = <NodeType as Render<RooibosDom>>::State;

    fn build(self) -> Self::State {
        self.node_type.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.node_type.rebuild(state);
    }
}

struct RenderProps<'a> {
    buf: &'a mut Buffer,
    window: Rect,
    key: DomNodeKey,
    dom_nodes: &'a NodeTree,
    dom_state: &'a mut DomState,
}

impl DomNodeInner {
    fn render(&self, props: RenderProps) {
        let RenderProps {
            buf,
            window,
            key,
            dom_nodes,
            dom_state,
        } = props;

        let prev_rect = *self.rect.borrow();
        let rect = dom_nodes.rect(key);

        if self.focusable {
            dom_state.add_focusable(key);
        }

        if self.clear {
            Clear.render(rect, buf);
        }

        match &self.node_type {
            NodeType::Layout => {
                if let Some(block) = &self.block {
                    block.render_ref(rect, buf);
                };

                self.renderable_children(dom_nodes).for_each(|key| {
                    dom_nodes[*key].render(RenderProps {
                        buf,
                        window,
                        key: *key,
                        dom_nodes,
                        dom_state,
                    });
                });
            }
            NodeType::Widget(widget) => {
                // prevent panic if the calculated rect overflows the window area
                let rect = rect.clamp(window);
                widget.render(rect, buf);
            }
            NodeType::Placeholder => {}
        }
        *self.rect.borrow_mut() = rect;
        if rect != prev_rect {
            if let Some(on_size_change) = &dom_nodes[key].event_handlers.on_size_change {
                on_size_change.borrow_mut()(rect);
            }
        }
    }

    fn renderable_children<'a>(
        &'a self,
        dom_nodes: &'a NodeTree,
    ) -> impl Iterator<Item = &DomNodeKey> {
        self.children
            .iter()
            .filter(|c| !matches!(dom_nodes[**c].node_type, NodeType::Placeholder))
    }
}

#[derive(Clone, Debug, Default)]
pub struct DomNode {
    key: DomNodeKey,
    unmounted: Arc<AtomicBool>,
}

impl AsDomNode for DomNode {
    fn as_dom_node(&self) -> &DomNode {
        self
    }
}

impl PartialEq for DomNode {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for DomNode {}

impl Mountable<RooibosDom> for DomNode {
    fn unmount(&mut self) {
        unmount_child(self.key(), false);
        self.unmounted.store(true, Ordering::SeqCst);
    }

    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        RooibosDom::insert_node(parent, self, marker);
        self.unmounted.store(false, Ordering::SeqCst);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        if let Some(parent) = RooibosDom::get_parent(self) {
            child.mount(&parent, Some(self));
            return true;
        }
        false
    }
}

impl Drop for DomNode {
    fn drop(&mut self) {
        // The thread-local may already have been destroyed
        // We need to check using try_with to prevent a panic here
        if self.unmounted.load(Ordering::SeqCst) && DOM_NODES.try_with(|_| {}).is_ok() {
            let contains_key = with_nodes(|nodes| nodes.contains_key(self.key));
            if contains_key {
                unmount_child(self.key, true);
            }
        }
    }
}

impl DomNode {
    pub(crate) fn from_existing(key: DomNodeKey, unmounted: Arc<AtomicBool>) -> Self {
        Self { key, unmounted }
    }

    pub(crate) fn placeholder() -> Self {
        let unmounted = Arc::new(AtomicBool::new(false));
        let inner = DomNodeInner {
            name: "placeholder".to_string(),
            node_type: NodeType::Placeholder,
            original_display: taffy::Display::None,
            unmounted: unmounted.clone(),
            ..Default::default()
        };
        let key = with_nodes_mut(|n| {
            let key = n.insert(inner);
            n.update_layout(key, |style| {
                style.display = taffy::Display::None;
            });
            key
        });
        Self { key, unmounted }
    }

    pub(crate) fn widget(widget: DomWidgetNode) -> Self {
        let unmounted = Arc::new(AtomicBool::new(false));
        let inner = DomNodeInner {
            name: widget.widget_type.clone(),
            node_type: NodeType::Widget(widget),
            original_display: taffy::Display::Block,
            unmounted: unmounted.clone(),
            ..Default::default()
        };
        let key = with_nodes_mut(|n| n.insert(inner));
        Self { key, unmounted }
    }

    pub(crate) fn flex_row() -> Self {
        let unmounted = Arc::new(AtomicBool::new(false));
        let inner = DomNodeInner {
            name: "flex_row".to_string(),
            node_type: NodeType::Layout,
            original_display: taffy::Display::Flex,
            unmounted: unmounted.clone(),
            ..Default::default()
        };
        let key = with_nodes_mut(|n| {
            let key = n.insert(inner);
            n.update_layout(key, |style| {
                style.display = taffy::Display::Flex;
                style.flex_direction = taffy::FlexDirection::Row;
            });
            key
        });
        Self { key, unmounted }
    }

    pub(crate) fn div() -> Self {
        let unmounted = Arc::new(AtomicBool::new(false));
        let inner = DomNodeInner {
            name: "div".to_string(),
            node_type: NodeType::Layout,
            original_display: taffy::Display::Block,
            unmounted: unmounted.clone(),
            ..Default::default()
        };
        let key = with_nodes_mut(|n| {
            let key = n.insert(inner);
            n.update_layout(key, |style| {
                style.display = taffy::Display::Block;
            });
            key
        });
        Self { key, unmounted }
    }

    pub(crate) fn flex_col() -> Self {
        let unmounted = Arc::new(AtomicBool::new(false));
        let inner = DomNodeInner {
            name: "flex_col".to_string(),
            node_type: NodeType::Layout,
            original_display: taffy::Display::Flex,
            unmounted: unmounted.clone(),
            ..Default::default()
        };
        let key = with_nodes_mut(|n| {
            let key = n.insert(inner);
            n.update_layout(key, |style| {
                style.display = taffy::Display::Flex;
                style.flex_direction = taffy::FlexDirection::Column;
            });
            key
        });
        Self { key, unmounted }
    }

    pub(crate) fn replace_node(&mut self, node: &DomNode) {
        with_nodes_mut(|nodes| {
            nodes.replace_node(self.key, node.key);
        });
        unmount_child(self.key, true);

        self.key = node.key;
    }

    pub(crate) fn replace_widget(&self, widget: DomWidgetNode) {
        let inner = DomNodeInner {
            name: widget.widget_type.clone(),
            node_type: NodeType::Widget(widget),
            parent: self.get_parent_key(),
            ..Default::default()
        };
        with_nodes_mut(|n| n.replace_inner(self.key, inner));
    }

    pub(crate) fn update_event_handlers<F>(&self, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        with_nodes_mut(|n| n.update_event_handlers(self.key, update));
    }

    pub(crate) fn set_id(&self, id: impl Into<NodeId>) {
        with_nodes_mut(|n| {
            n.set_id(self.key, id);
        });
    }

    pub(crate) fn set_z_index(&self, z_index: i32) {
        with_nodes_mut(|n| {
            n.set_z_index(self.key, z_index);
        });
    }

    pub(crate) fn set_class(&self, class: impl Into<String>) {
        with_nodes_mut(|n| {
            n.set_class(self.key, class);
        });
    }

    pub(crate) fn key(&self) -> DomNodeKey {
        self.key
    }

    pub(crate) fn get_parent_key(&self) -> Option<DomNodeKey> {
        with_nodes(|n| n[self.key].parent)
    }

    pub(crate) fn get_parent(&self) -> Option<DomNode> {
        let parent_key = with_nodes(|n| n[self.key].parent);

        parent_key.map(|k| {
            let unmounted = with_nodes(|n| n[k].unmounted.clone());
            DomNode::from_existing(k, unmounted)
        })
    }

    pub(crate) fn insert_before(&self, child: &DomNode, reference: Option<&DomNode>) {
        with_nodes_mut(|nodes| nodes.insert_before(self.key, child.key, reference.map(|r| r.key)))
    }

    pub(crate) fn render(&self, buf: &mut Buffer, rect: Rect) {
        with_nodes(|nodes| {
            with_state_mut(|state| {
                state.clear_focusables();

                nodes[self.key].render(RenderProps {
                    buf,
                    window: rect,
                    key: self.key,
                    dom_nodes: nodes,
                    dom_state: state,
                });
            });
        });
    }
}

impl Render<RooibosDom> for DomNode {
    type State = (DomNode, <NodeType as Render<RooibosDom>>::State);

    fn build(self) -> Self::State {
        let state = with_nodes(|n| n[self.key].node_type.clone().build());
        (self, state)
    }

    fn rebuild(self, (_node, ref mut node_type_state): &mut Self::State) {
        with_nodes(|n| n[self.key].node_type.clone().rebuild(node_type_state));
    }
}
