use core::fmt::Debug;
use std::cell::RefCell;
use std::fmt::{self};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use educe::Educe;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Block, Clear, Widget, WidgetRef};
#[cfg(feature = "effects")]
use tachyonfx::{EffectRenderer, Shader};
use terminput::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, ScrollDirection};

use super::node_tree::{DomNodeKey, NodeTree};
use super::{refresh_dom, unmount_child};
use crate::events::{
    BlurEvent, EventData, EventHandle, EventHandlers, FocusEvent, IntoClickHandler, IntoKeyHandler,
    dispatch_event, reset_mouse_position,
};
use crate::widgets::Role;
use crate::{
    DomWidgetNode, MatchBehavior, next_node_id, tree_is_accessible, with_nodes, with_nodes_mut,
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
            NodeIdInner::Auto(val) => fmt::Display::fmt(&val, f),
            NodeIdInner::Manual(id) => fmt::Display::fmt(&id, f),
        }
    }
}

impl From<String> for NodeId {
    fn from(val: String) -> Self {
        Self::new(val)
    }
}

impl From<&str> for NodeId {
    fn from(val: &str) -> Self {
        Self::new(val)
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
    pub(crate) fn from_node(key: DomNodeKey, node: &NodeProperties) -> Self {
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

    pub fn find<F>(&self, mut f: F) -> Option<DomNodeRepr>
    where
        F: FnMut(&DomNodeRepr) -> bool,
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
        F: FnMut(&DomNodeRepr) -> bool,
    {
        self.find(f).unwrap()
    }

    pub fn find_all<F>(&self, mut f: F) -> Vec<DomNodeRepr>
    where
        F: FnMut(&DomNodeRepr) -> bool,
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

    pub fn is_focused(&self) -> bool {
        with_nodes(|nodes| {
            let found_node = nodes
                .iter_nodes()
                .find_map(|(key, _)| if key == self.key { Some(key) } else { None });
            nodes.focused_key() == found_node
        })
    }
}

#[cfg(feature = "effects")]
#[derive(Debug, Clone)]
pub struct EffectProperties {
    effect: tachyonfx::Effect,
    last_tick: std::time::Instant,
}

#[cfg(feature = "effects")]
impl EffectProperties {
    pub(crate) fn new(effect: tachyonfx::Effect) -> Self {
        Self {
            effect,
            last_tick: std::time::Instant::now(),
        }
    }

    fn tick(&mut self) -> tachyonfx::Duration {
        let elapsed = self.last_tick.elapsed();
        self.last_tick = std::time::Instant::now();
        elapsed.into()
    }
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub struct NodeProperties {
    pub(crate) node_type: NodeType,
    pub(crate) name: String,
    pub(crate) original_display: taffy::Display,
    pub(crate) children: Vec<DomNodeKey>,
    pub(crate) parent: Option<DomNodeKey>,
    pub(crate) id: Option<NodeId>,
    pub(crate) class: Option<String>,
    focusable: bool,
    #[educe(Debug(ignore))]
    pub(crate) event_handlers: EventHandlers,
    pub(crate) rect: Rc<RefCell<Rect>>,
    pub(crate) z_index: Option<i32>,
    pub(crate) block: Option<Block<'static>>,
    pub(crate) clear: bool,
    pub(crate) scroll_offset: Position,
    pub(crate) ancestor_scroll_offset: Position,
    pub(crate) max_scroll_offset: Position,
    #[cfg(feature = "effects")]
    pub(crate) effects: Option<RefCell<EffectProperties>>,
    #[educe(Default = true)]
    enabled: bool,
    #[educe(Default = true)]
    parent_enabled: bool,
    pub(crate) unmounted: Arc<AtomicBool>,
    #[educe(Default = AtomicBool::new(true))]
    visible: AtomicBool,
}

struct RenderProps<'a, 'b> {
    frame: &'a mut Frame<'b>,
    window: Rect,
    parent_bounds: Rect,
    parent_scroll_offset: Position,
    key: DomNodeKey,
    dom_nodes: &'a NodeTree,
}

impl NodeProperties {
    pub(crate) fn enabled(&self) -> bool {
        // If a node is disabled, all children should also be disabled
        self.enabled && self.parent_enabled
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn focusable(&self) -> bool {
        self.focusable && self.enabled()
    }

    pub(crate) fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    pub(crate) fn set_parent_enabled(&mut self, enabled: bool) {
        self.parent_enabled = enabled;
    }

    pub(crate) fn visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub(crate) fn position(&self) -> Rect {
        let mut rect = *self.rect.borrow();
        rect.x = rect
            .x
            .saturating_sub(self.scroll_offset.x)
            .saturating_sub(self.ancestor_scroll_offset.x);
        rect.y = rect
            .y
            .saturating_sub(self.scroll_offset.y)
            .saturating_sub(self.ancestor_scroll_offset.y);
        rect
    }

    pub(crate) fn scroll(&mut self, direction: ScrollDirection) -> taffy::Point<i32> {
        let delta = direction.delta();
        let mut x = self.scroll_offset.x as i32 - delta.x;
        let mut y = self.scroll_offset.y as i32 - delta.y;

        x = x.min(self.max_scroll_offset.x as i32).max(0);
        y = y.min(self.max_scroll_offset.y as i32).max(0);

        let x_change = x - self.scroll_offset.x as i32;
        let y_change = y - self.scroll_offset.y as i32;
        self.scroll_offset.x = x as u16;
        self.scroll_offset.y = y as u16;
        taffy::Point {
            x: x_change,
            y: y_change,
        }
    }

    pub(crate) fn update_ancestor_scroll_offsets(&mut self, change: taffy::Point<i32>) {
        self.ancestor_scroll_offset.x =
            (self.ancestor_scroll_offset.x as i32 + change.x).max(0) as u16;
        self.ancestor_scroll_offset.y =
            (self.ancestor_scroll_offset.y as i32 + change.y).max(0) as u16;
    }

    fn render(&self, props: RenderProps) {
        let RenderProps {
            frame,
            window,
            key,
            parent_bounds,
            parent_scroll_offset,
            dom_nodes,
        } = props;

        let prev_rect = *self.rect.borrow();
        let content_rect = dom_nodes.rect(key);

        if self.focusable {
            dom_nodes.add_focusable(key);
        }

        let render_bounds = content_rect.render_bounds();

        let height_above_parent = (render_bounds.y + render_bounds.height)
            .saturating_sub(parent_bounds.y + parent_bounds.height)
            .saturating_sub(parent_scroll_offset.y);

        let mut temp_buf = if content_rect.can_scroll() || height_above_parent > 0 {
            Some(Buffer::empty(Rect::default()))
        } else {
            None
        };
        if let Some(temp_buf) = &mut temp_buf {
            content_rect.resize_for_render(temp_buf);
        }

        // if height_above_parent > 0 {
        //     print!("");
        // }

        let visible_bounds = content_rect.visible_bounds();
        let needs_render = (visible_bounds.x
            < parent_bounds.x + parent_bounds.width + parent_scroll_offset.x)
            && (visible_bounds.y < parent_bounds.y + parent_bounds.height + parent_scroll_offset.y);

        if self.clear {
            Clear.render(render_bounds, frame.buffer_mut());
        }

        if let NodeType::Widget(widget) = &self.node_type {
            widget.recompute_done();
        }
        if needs_render {
            self.visible.store(true, Ordering::Relaxed);
            match &self.node_type {
                NodeType::Layout => {
                    if let Some(temp_buf) = &mut temp_buf {
                        // let mut temp_buf = dom_nodes.temp_buf.borrow_mut();

                        temp_buf.area.x = visible_bounds.x;
                        temp_buf.area.y = visible_bounds.y;
                        // temp_buf.reset();

                        mem::swap(frame.buffer_mut(), temp_buf);

                        let child_bounds = content_rect.child_bounds();
                        self.children.iter().for_each(|key| {
                            dom_nodes[*key].render(RenderProps {
                                frame,
                                window,
                                parent_bounds: child_bounds,
                                parent_scroll_offset: content_rect.scroll_offset(),
                                key: *key,
                                dom_nodes,
                            });
                        });
                        content_rect.apply_scroll(frame.buffer_mut().area, frame.buffer_mut());

                        // let mut temp_buf = dom_nodes.temp_buf.borrow_mut();
                        mem::swap(frame.buffer_mut(), temp_buf);

                        let buf = frame.buffer_mut();
                        for row in child_bounds.rows().take(
                            (child_bounds.height.saturating_sub(height_above_parent)) as usize,
                        ) {
                            for col in row.columns() {
                                let pos: Position = col.into();
                                if pos.x < buf.area.x + buf.area.width
                                    && pos.y < buf.area.y + buf.area.height
                                {
                                    buf[pos] = temp_buf[pos].clone();
                                }
                            }
                        }
                        self.render_block(render_bounds, frame.buffer_mut());
                    } else {
                        self.children.iter().for_each(|key| {
                            dom_nodes[*key].render(RenderProps {
                                frame,
                                window,
                                parent_bounds: content_rect.child_bounds(),
                                parent_scroll_offset: content_rect.scroll_offset(),
                                key: *key,
                                dom_nodes,
                            });
                        });
                        if let Some(block) = &self.block {
                            block.render_ref(render_bounds, frame.buffer_mut());
                        };
                    }
                }
                NodeType::Widget(widget) => {
                    self.visible.store(true, Ordering::Relaxed);
                    let inner = content_rect.inner_bounds();
                    if let Some(temp_buf) = &mut temp_buf {
                        // let mut temp_buf = dom_nodes.temp_buf.borrow_mut();
                        // temp_buf.reset();
                        temp_buf.area.x = inner.x;
                        temp_buf.area.y = inner.y;

                        mem::swap(frame.buffer_mut(), temp_buf);

                        widget.render(inner, frame);

                        content_rect.apply_scroll(frame.buffer_mut().area, frame.buffer_mut());

                        mem::swap(frame.buffer_mut(), temp_buf);

                        for row in inner
                            .rows()
                            .take((inner.height.saturating_sub(height_above_parent)) as usize)
                        {
                            for col in row.columns() {
                                let pos: Position = col.into();
                                frame.buffer_mut()[pos] = temp_buf[pos].clone();
                            }
                        }
                        self.render_block(render_bounds, frame.buffer_mut());
                    } else {
                        widget.render(inner, frame);
                        self.render_block(render_bounds, frame.buffer_mut());
                    }
                }
                NodeType::Placeholder => {}
            }
            #[cfg(feature = "effects")]
            if let Some(effects) = &self.effects {
                let mut effects = effects.borrow_mut();

                if effects.effect.running() {
                    let tick = effects.tick();

                    frame.render_effect(&mut effects.effect, render_bounds, tick);
                    refresh_dom();
                }
            }
        } else {
            self.visible.store(false, Ordering::Relaxed);
        }

        *self.rect.borrow_mut() = render_bounds;
        if render_bounds != prev_rect {
            if let Some(on_size_change) = &dom_nodes[key].event_handlers.on_size_change {
                on_size_change.borrow_mut()(render_bounds);
            }
        }
    }

    fn render_block(&self, bounds: Rect, buf: &mut Buffer) {
        if let Some(block) = &self.block {
            block.render_ref(bounds, buf);
        };
    }

    pub(crate) fn replace_with(&mut self, new: &NodeProperties) {
        // This is annoyingly verbose, but we use destructuring here to ensure we account for
        // any new properties that get added to DomNodeInner
        let NodeProperties {
            node_type,
            name,
            children: _children,
            parent: _parent,
            id,
            class,
            focusable,
            event_handlers,
            rect,
            original_display,
            block,
            clear,
            enabled,
            scroll_offset,
            ancestor_scroll_offset: _ancestor_scroll_offset,
            max_scroll_offset,
            #[cfg(feature = "effects")]
            effects,
            z_index: _z_index,
            unmounted: _unmounted,
            parent_enabled: _parent_enabled,
            visible: _visible,
        } = new;

        let name = name.clone();
        let node_type = node_type.clone();
        let focusable = *focusable;
        let id = id.clone();
        let class = class.clone();
        let event_handlers = event_handlers.clone();
        let rect = rect.clone();
        let original_display = *original_display;
        let block = block.clone();
        let clear = *clear;
        let enabled = *enabled;
        let scroll_offset = *scroll_offset;
        let max_scroll_offset = *max_scroll_offset;
        #[cfg(feature = "effects")]
        let effects = effects.clone();

        self.name = name;
        self.node_type = node_type;
        self.focusable = focusable;
        self.id = id;
        self.class = class;
        self.event_handlers = event_handlers;
        self.rect = rect;
        self.original_display = original_display;
        self.block = block;
        self.clear = clear;
        self.enabled = enabled;
        self.scroll_offset = scroll_offset;
        self.max_scroll_offset = max_scroll_offset;
        #[cfg(feature = "effects")]
        {
            self.effects = effects;
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
        let inner = NodeProperties {
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
        let inner = NodeProperties {
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
        let inner = NodeProperties {
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
        let inner = NodeProperties {
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
        let inner = NodeProperties {
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
        let inner = NodeProperties {
            name: widget.widget_type.clone(),
            node_type: NodeType::Widget(widget),
            parent: self.get_parent_key(),
            ..Default::default()
        };
        with_nodes_mut(|n| n.replace_inner(self.key, inner));
    }

    fn update_event_handlers<F>(&self, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        with_nodes_mut(|n| n.update_event_handlers(self.key, update));
    }

    pub fn force_recompute_layout(&self) {
        with_nodes_mut(|n| n.force_recompute_layout(self.key));
    }

    pub fn on_key_down<H>(self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        self.update_event_handlers(|h| h.on_key_down(handler));
        self
    }

    pub fn on_key_up<H>(self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        self.update_event_handlers(|h| h.on_key_up(handler));
        self
    }

    pub fn on_click<H>(self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.update_event_handlers(|h| h.on_click(handler));
        self
    }

    pub fn on_right_click<H>(self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.update_event_handlers(|h| h.on_right_click(handler));
        self
    }

    pub fn on_middle_click<H>(self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.update_event_handlers(|h| h.on_middle_click(handler));
        self
    }

    pub fn on_paste<F>(self, handler: F) -> Self
    where
        F: FnMut(String, EventData, EventHandle) + 'static,
    {
        self.update_event_handlers(|h| h.on_paste(handler));
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

    pub fn on_mouse_enter<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.update_event_handlers(|h| h.on_mouse_enter(handler));
        self
    }

    pub fn on_mouse_leave<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.update_event_handlers(|h| h.on_mouse_leave(handler));
        self
    }

    pub fn on_size_change<F>(self, handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.update_event_handlers(|h| h.on_size_change(handler));
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

    #[cfg(feature = "effects")]
    pub fn effect(self, effect: tachyonfx::Effect) -> Self {
        with_nodes_mut(|n| {
            n.set_effect(self.key, effect);
        });
        self
    }

    pub fn z_index(self, z_index: i32) -> Self {
        with_nodes_mut(|n| {
            n.set_z_index(self.key, z_index);
        });
        self
    }

    pub fn class(self, class: impl Into<String>) -> Self {
        with_nodes_mut(|n| {
            n.set_class(self.key, class);
        });
        self
    }

    pub fn focusable(self, focusable: bool) -> Self {
        with_nodes_mut(|nodes| nodes.set_focusable(self.key, focusable));
        self
    }

    pub fn get_key(&self) -> DomNodeKey {
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

    pub fn render(&self, window: Rect, frame: &mut Frame) {
        with_nodes(|nodes| {
            nodes[self.key].render(RenderProps {
                frame,
                window,
                parent_bounds: window,
                parent_scroll_offset: Position::ORIGIN,
                key: self.key,
                dom_nodes: nodes,
                // using_temp_buf: false,
            });
        });
        reset_mouse_position();
    }
}
