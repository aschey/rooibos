use core::fmt::Debug;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use derivative::Derivative;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::widgets::{Block, WidgetRef};
use slotmap::{new_key_type, SlotMap};
use tachys::reactive_graph::RenderEffectState;
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};
use terminput::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::document_fragment::DocumentFragment;
use super::dom_state::{self, DomState};
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

impl DomNodeKey {
    pub(crate) fn traverse<F, T>(&self, f: F, stop_on_first_match: bool) -> Vec<T>
    where
        F: FnMut(DomNodeKey, &DomNodeInner) -> Option<T> + Clone,
    {
        let mut out_list = vec![];
        self.traverse_inner(f, &mut out_list, stop_on_first_match);
        out_list
    }

    fn traverse_inner<F, T>(&self, mut f: F, out_list: &mut Vec<T>, stop_on_first_match: bool)
    where
        F: FnMut(DomNodeKey, &DomNodeInner) -> Option<T> + Clone,
    {
        if let Some(out) = with_nodes(|nodes| f(*self, &nodes[*self])) {
            out_list.push(out);
            if stop_on_first_match {
                return;
            }
        }
        let children = with_nodes(|nodes| nodes[*self].children.clone());
        for child in children {
            child.traverse_inner(f.clone(), out_list, stop_on_first_match);
        }
    }
}

pub(crate) struct NodeTypeStructure {
    pub(crate) name: &'static str,
    pub(crate) attrs: Option<String>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct LayoutProps {
    pub direction: Direction,
    pub flex: Flex,
    pub margin: u16,
    pub spacing: u16,
    pub block: Option<Block<'static>>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum NodeType {
    Layout(Rc<RefCell<LayoutProps>>),
    Overlay,
    Absolute(Rc<RefCell<(u16, u16)>>),
    Widget(DomWidgetNode),
    Placeholder,
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
                }
            }
            NodeType::Absolute(pos) => {
                let (x, y) = *pos.borrow();
                NodeTypeStructure {
                    name: "Absolute",
                    attrs: Some(format!("x={x} y={y}")),
                }
            }
            NodeType::Overlay => NodeTypeStructure {
                name: "Overlay",
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
            NodeType::Absolute(pos) => {
                let (x, y) = *pos.borrow();
                write!(f, "Absolute(x={x} y={y})")
            }
            NodeType::Overlay => write!(f, "Overlay"),
            NodeType::Widget(widget) => write!(f, "Widget({widget:?})"),
            NodeType::Placeholder => write!(f, "Placeholder"),
        }
    }
}

impl Render<RooibosDom> for NodeType {
    type State = Option<RenderEffectState<()>>;

    fn build(self) -> Self::State {
        match self {
            NodeType::Layout(_) => None,
            NodeType::Overlay => None,
            NodeType::Absolute(_) => None,
            NodeType::Widget(node) => Some(node.build()),
            NodeType::Placeholder => None,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match self {
            NodeType::Layout(_) => {}
            NodeType::Overlay => {}
            NodeType::Absolute(_) => {}
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
    Layout(LayoutProps),
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
                NodeType::Layout(props) => NodeTypeRepr::Layout(props.borrow().clone()),
                NodeType::Overlay => NodeTypeRepr::Overlay,
                NodeType::Absolute(_) => NodeTypeRepr::Absolute,
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
                .iter()
                .find_map(|(key, _)| if key == self.key { Some(key) } else { None })
        })
        .unwrap();
        dom_state::set_focused(found_node);
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct DomNodeInner {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) constraint: Rc<RefCell<Constraint>>,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) class: Option<String>,
    pub(crate) focusable: Rc<RefCell<bool>>,
    #[derivative(Debug = "ignore")]
    pub(crate) event_handlers: EventHandlers,
    pub(crate) rect: Rc<RefCell<Rect>>,
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
    rect: Rect,
    window: Rect,
    key: DomNodeKey,
    dom_nodes: &'a SlotMap<DomNodeKey, DomNodeInner>,
    dom_state: &'a mut DomState,
    parent_layout: Layout,
}

impl DomNodeInner {
    fn render(&self, props: RenderProps) {
        let RenderProps {
            buf,
            rect,
            window,
            key,
            dom_nodes,
            dom_state,
            parent_layout,
        } = props;
        let prev_rect = *self.rect.borrow();
        *self.rect.borrow_mut() = rect;

        if *self.focusable.borrow() {
            dom_state.add_focusable(key);
        }

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
                    block.render_ref(rect, buf);
                };

                let constraints = self
                    .layout_children(dom_nodes)
                    .map(|key| *dom_nodes[*key].constraint.borrow());
                let layout = Layout::default()
                    .direction(direction)
                    .flex(flex)
                    .margin(margin)
                    .spacing(spacing)
                    .constraints(constraints);

                let chunks = layout.split(rect);
                self.layout_children(dom_nodes)
                    .zip(chunks.iter())
                    .for_each(|(key, chunk)| {
                        dom_nodes[*key].render(RenderProps {
                            buf,
                            rect: *chunk,
                            window,
                            key: *key,
                            dom_nodes,
                            dom_state,
                            parent_layout: Layout::default(),
                        });
                    });
                self.absolute_children(dom_nodes).for_each(|key| {
                    dom_nodes[*key].render(RenderProps {
                        buf,
                        rect: window,
                        window,
                        key: *key,
                        dom_nodes,
                        dom_state,
                        parent_layout: Layout::default(),
                    });
                });
            }
            NodeType::Overlay => {
                let parent_layout = parent_layout.constraints([*self.constraint.borrow()]);
                let chunks = parent_layout.split(rect);
                self.renderable_children(dom_nodes).for_each(|key| {
                    dom_nodes[*key].render(RenderProps {
                        buf,
                        rect: chunks[0],
                        window,
                        key: *key,
                        dom_nodes,
                        dom_state,
                        parent_layout: parent_layout.clone(),
                    });
                });
            }
            NodeType::Absolute(pos) => {
                let (x, y) = *pos.borrow();
                let rect = Rect::new(x, y, window.width - x, window.height - y);
                self.renderable_children(dom_nodes).for_each(|key| {
                    dom_nodes[*key].render(RenderProps {
                        buf,
                        rect,
                        window,
                        key: *key,
                        dom_nodes,
                        dom_state,
                        parent_layout: parent_layout.clone(),
                    });
                });
            }
            NodeType::Widget(widget) => {
                let parent_layout = parent_layout.constraints([*self.constraint.borrow()]);
                let chunks = parent_layout.split(rect);
                widget.render(chunks[0], buf);
                *self.rect.borrow_mut() = chunks[0];
            }
            NodeType::Placeholder => {}
        }

        let rect = *self.rect.borrow();
        if rect != prev_rect {
            if let Some(on_size_change) = &dom_nodes[key].event_handlers.on_size_change {
                on_size_change.borrow_mut()(rect);
            }
        }
    }

    fn renderable_children<'a>(
        &'a self,
        dom_nodes: &'a SlotMap<DomNodeKey, DomNodeInner>,
    ) -> impl Iterator<Item = &DomNodeKey> {
        self.children
            .iter()
            .filter(|c| !matches!(dom_nodes[**c].node_type, NodeType::Placeholder))
    }

    fn layout_children<'a>(
        &'a self,
        dom_nodes: &'a SlotMap<DomNodeKey, DomNodeInner>,
    ) -> impl Iterator<Item = &DomNodeKey> {
        self.renderable_children(dom_nodes)
            .filter(|c| !matches!(dom_nodes[**c].node_type, NodeType::Absolute(_)))
    }

    fn absolute_children<'a>(
        &'a self,
        dom_nodes: &'a SlotMap<DomNodeKey, DomNodeInner>,
    ) -> impl Iterator<Item = &DomNodeKey> {
        self.renderable_children(dom_nodes)
            .filter(|c| matches!(dom_nodes[**c].node_type, NodeType::Absolute(_)))
    }
}

#[derive(Clone, Debug)]
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
    pub(crate) fn placeholder() -> Self {
        let inner = DomNodeInner {
            name: "Placeholder".to_string(),
            node_type: NodeType::Placeholder,
            constraint: Default::default(),
            children: vec![],
            parent: None,
            focusable: Default::default(),
            id: None,
            class: None,
            event_handlers: Default::default(),
            rect: Default::default(),
        };
        let key = with_nodes_mut(|n| n.insert(inner));
        Self {
            key,
            unmounted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn from_fragment(fragment: DocumentFragment) -> Self {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: Rc::new(RefCell::new(fragment.constraint)),
            children: vec![],
            parent: None,
            focusable: Rc::new(RefCell::new(fragment.focusable)),
            id: fragment.id,
            class: fragment.class,
            event_handlers: fragment.event_handlers,
            rect: Rc::new(RefCell::new(Rect::default())),
        };
        let key = with_nodes_mut(|n| n.insert(inner));
        Self {
            key,
            unmounted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn replace_fragment(&self, fragment: DocumentFragment) {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: Rc::new(RefCell::new(fragment.constraint)),
            children: vec![],
            parent: self.get_parent_key(),
            focusable: Rc::new(RefCell::new(fragment.focusable)),
            id: fragment.id,
            class: fragment.class,
            event_handlers: fragment.event_handlers,
            rect: Rc::new(RefCell::new(Rect::default())),
        };
        with_nodes_mut(|n| n[self.key] = inner);
    }

    pub(crate) fn replace_node(&mut self, node: &DomNode) {
        with_nodes_mut(|nodes| {
            // This is annoyingly verbose, but we use destructuring here to ensure we account for
            // any new properties that get added to DomNodeInner
            let DomNodeInner {
                node_type,
                name,
                constraint,
                children: _children,
                parent: _parent,
                id,
                class,
                focusable,
                event_handlers,
                rect,
            } = &nodes[self.key];
            let name = name.clone();
            let node_type = node_type.clone();
            let constraint = constraint.clone();
            let focusable = focusable.clone();
            let id = id.clone();
            let class = class.clone();
            let event_handlers = event_handlers.clone();
            let rect = rect.clone();

            let new = &mut nodes[node.key];

            new.name = name;
            new.node_type = node_type;
            new.constraint = constraint;
            new.focusable = focusable;
            new.id = id;
            new.class = class;
            new.event_handlers = event_handlers;
            new.rect = rect;
        });
        unmount_child(self.key, true);

        self.key = node.key;
    }

    pub(crate) fn set_constraint(&self, constraint: Rc<RefCell<Constraint>>) {
        with_nodes_mut(|n| n[self.key].constraint = constraint);
    }

    pub(crate) fn set_focusable(&self, focusable: Rc<RefCell<bool>>) {
        with_nodes_mut(|n| n[self.key].focusable = focusable);
    }

    pub(crate) fn update_event_handlers<F>(&self, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        with_nodes_mut(|n| n[self.key].event_handlers = update(n[self.key].event_handlers.clone()));
    }

    pub(crate) fn set_id(&self, id: impl Into<NodeId>) {
        with_nodes_mut(|n| {
            n[self.key].id = Some(id.into());
            *n[self.key].focusable.borrow_mut() = true;
        });
    }

    pub(crate) fn set_class(&self, class: impl Into<String>) {
        with_nodes_mut(|n| {
            n[self.key].class = Some(class.into());
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
        with_nodes(|n| {
            n[self.key].parent.map(|p| DomNode {
                key: p,
                unmounted: Arc::new(AtomicBool::new(false)),
            })
        })
    }

    pub(crate) fn insert_before(&self, child: &DomNode, reference: Option<&DomNode>) {
        with_nodes_mut(|nodes| {
            if let Some(reference) = reference {
                if let Some(reference_pos) = nodes[self.key]
                    .children
                    .iter()
                    .position(|c| *c == reference.key)
                {
                    nodes[self.key].children.insert(reference_pos, child.key);
                    nodes[child.key].parent = Some(self.key);
                }
            } else {
                nodes[self.key].children.push(child.key);
                nodes[child.key].parent = Some(self.key);
            }
        })
    }

    pub(crate) fn render(&self, buf: &mut Buffer, rect: Rect) {
        with_nodes(|nodes| {
            with_state_mut(|state| {
                state.clear_focusables();
                let constraint = *nodes[self.key].constraint.borrow();

                nodes[self.key].render(RenderProps {
                    buf,
                    rect,
                    window: rect,
                    key: self.key,
                    dom_nodes: nodes,
                    dom_state: state,
                    parent_layout: Layout::default().constraints([constraint]),
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
