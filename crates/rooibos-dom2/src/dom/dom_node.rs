use core::fmt::Debug;
use std::cell::RefCell;
use std::fmt::{self};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use educe::Educe;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Clear, Widget, WidgetRef};
use terminput::{Event, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::node_tree::{DomNodeKey, NodeTree};
use super::unmount_child;
use crate::{
    dispatch_event, next_node_id, reset_mouse_position, tree_is_accessible, with_nodes,
    with_nodes_mut, BlurEvent, ClickEvent, DomWidgetNode, EventData, EventHandle, EventHandlers,
    FocusEvent, MatchBehavior, Role,
};

pub trait AsDomNode {
    fn as_dom_node(&self) -> &DomNode;
}

impl<T1, T2> AsDomNode for (T1, T2)
where
    T1: AsDomNode,
{
    fn as_dom_node(&self) -> &DomNode {
        self.0.as_dom_node()
    }
}

impl AsDomNode for Box<dyn AsDomNode> {
    fn as_dom_node(&self) -> &DomNode {
        (**self).as_dom_node()
    }
}

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
                MatchBehavior::StopOnFistMatch,
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
            MatchBehavior::ContinueOnMatch,
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
            MatchBehavior::StopOnFistMatch,
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
            MatchBehavior::ContinueOnMatch,
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
            MatchBehavior::ContinueOnMatch,
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
        dispatch_event(event);
    }

    pub fn focus(&self) {
        with_nodes_mut(|nodes| {
            let found_node = nodes
                .iter_nodes()
                .find_map(|(key, _)| if key == self.key { Some(key) } else { None });
            if let Some(found_node) = found_node {
                nodes.set_focused(Some(found_node));
            }
        });
    }
}

#[derive(Educe, Default)]
#[educe(Debug)]
pub struct DomNodeInner {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) original_display: taffy::Display,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) class: Option<String>,
    pub(crate) focusable: bool,
    #[educe(Debug(ignore))]
    pub(crate) event_handlers: EventHandlers,
    pub(crate) rect: Rc<RefCell<Rect>>,
    pub(crate) z_index: Option<i32>,
    pub(crate) block: Option<Block<'static>>,
    pub(crate) clear: bool,
    pub(crate) disabled: bool,
    pub(crate) unmounted: Arc<AtomicBool>,
}

struct RenderProps<'a> {
    buf: &'a mut Buffer,
    window: Rect,
    key: DomNodeKey,
    dom_nodes: &'a NodeTree,
}

impl DomNodeInner {
    fn render(&self, props: RenderProps) {
        let RenderProps {
            buf,
            window,
            key,
            dom_nodes,
        } = props;

        let prev_rect = *self.rect.borrow();
        let mut rect = dom_nodes.rect(key);

        if self.focusable {
            dom_nodes.add_focusable(key);
        }

        if self.clear {
            Clear.render(rect, buf);
        }

        match &self.node_type {
            NodeType::Layout => {
                if let Some(block) = &self.block {
                    block.render_ref(rect, buf);
                };

                self.children.iter().for_each(|key| {
                    dom_nodes[*key].render(RenderProps {
                        buf,
                        window,
                        key: *key,
                        dom_nodes,
                    });
                });
            }
            NodeType::Widget(widget) => {
                // if the widget width == the window width, there's probably no explicit width
                // subtract the extra to prevent clamp from removing any margins
                if rect.width == window.width {
                    rect.width -= rect.x;
                }
                if rect.height == window.height {
                    rect.height -= rect.y;
                }
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

impl Drop for DomNode {
    fn drop(&mut self) {
        // The thread-local may already have been destroyed
        // We need to check to prevent a panic here
        if self.unmounted.load(Ordering::SeqCst) && tree_is_accessible() {
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

    pub fn placeholder() -> Self {
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

    pub fn widget(widget: DomWidgetNode) -> Self {
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

    pub fn flex_row() -> Self {
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

    pub fn div() -> Self {
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

    pub fn flex_col() -> Self {
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

    pub fn replace_node(&mut self, node: &DomNode) {
        with_nodes_mut(|nodes| {
            nodes.replace_node(self.key, node.key);
        });
        unmount_child(self.key, true);

        self.key = node.key;
    }

    pub fn replace_widget(&self, widget: DomWidgetNode) {
        let inner = DomNodeInner {
            name: widget.widget_type.clone(),
            node_type: NodeType::Widget(widget),
            parent: self.get_parent_key(),
            ..Default::default()
        };
        with_nodes_mut(|n| n.replace_inner(self.key, inner));
    }

    pub fn update_event_handlers<F>(&self, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        with_nodes_mut(|n| n.update_event_handlers(self.key, update));
    }

    pub fn on_key_down<F>(self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData, &mut EventHandle) + 'static,
    {
        self.update_event_handlers(|h| h.on_key_down(handler));
        with_nodes_mut(|nodes| nodes.set_focusable(self.key, true));
        self
    }

    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: FnMut(ClickEvent, EventData, &mut EventHandle) + 'static,
    {
        self.update_event_handlers(|h| h.on_click(handler));
        with_nodes_mut(|nodes| nodes.set_focusable(self.key, true));
        self
    }

    pub fn on_focus<F>(self, handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.update_event_handlers(|h| h.on_focus(handler));
        self
    }

    pub fn on_blur<F>(self, handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.update_event_handlers(|h| h.on_blur(handler));
        self
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        with_nodes_mut(|n| {
            n.set_id(self.key, id);
        });
        self
    }

    pub fn block(self, block: Block<'static>) -> Self {
        with_nodes_mut(|n| {
            n.set_block(self.key, block);
        });
        self
    }

    pub fn set_z_index(&self, z_index: i32) {
        with_nodes_mut(|n| {
            n.set_z_index(self.key, z_index);
        });
    }

    pub fn set_class(&self, class: impl Into<String>) {
        with_nodes_mut(|n| {
            n.set_class(self.key, class);
        });
    }

    pub fn key(&self) -> DomNodeKey {
        self.key
    }

    pub(crate) fn get_parent_key(&self) -> Option<DomNodeKey> {
        with_nodes(|n| n[self.key].parent)
    }

    pub fn get_parent(&self) -> Option<DomNode> {
        let parent_key = with_nodes(|n| n[self.key].parent);

        parent_key.map(|k| {
            let unmounted = with_nodes(|n| n[k].unmounted.clone());
            DomNode::from_existing(k, unmounted)
        })
    }

    pub fn get_next_sibling(&self) -> Option<DomNode> {
        let parent_key = with_nodes(|n| n[self.key].parent);

        parent_key.and_then(|k| {
            let children_len = with_nodes(|n| n[k].children.len());
            let position =
                with_nodes(|n| n[k].children.iter().position(|c| *c == self.key)).unwrap();
            if position < children_len - 1 {
                let sibling = with_nodes(|n| n[k].children[position + 1]);
                let unmounted = with_nodes(|n| n[sibling].unmounted.clone());
                Some(DomNode::from_existing(sibling, unmounted))
            } else {
                None
            }
        })
    }

    pub fn get_first_child(&self) -> Option<DomNode> {
        let child_key = with_nodes(|n| n[self.key].children.first().cloned());

        child_key.map(|k| {
            let unmounted = with_nodes(|n| n[k].unmounted.clone());
            DomNode::from_existing(k, unmounted)
        })
    }

    pub fn insert_before<D1, D2>(&self, child: &D1, reference: Option<&D2>)
    where
        D1: AsDomNode,
        D2: AsDomNode,
    {
        with_nodes_mut(|nodes| {
            nodes.insert_before(
                self.key,
                child.as_dom_node().key,
                reference.map(|r| r.as_dom_node().key),
            )
        })
    }

    pub fn append<D1>(&self, child: &D1)
    where
        D1: AsDomNode,
    {
        self.insert_before::<D1, D1>(child, None)
    }

    pub fn render(&self, buf: &mut Buffer, rect: Rect) {
        with_nodes(|nodes| {
            nodes[self.key].render(RenderProps {
                buf,
                window: rect,
                key: self.key,
                dom_nodes: nodes,
            });
        });
        reset_mouse_position();
    }
}
