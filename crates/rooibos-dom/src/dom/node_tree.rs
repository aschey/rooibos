use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{cmp, mem};

use accesskit::TreeUpdate;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use slotmap::{SlotMap, new_key_type};
use taffy::{AvailableSpace, NodeId, Overflow, Point, Size, Style, TaffyTree};
use terminput::{KeyCode, KeyEvent, KeyEventKind, SHIFT, ScrollDirection, key};

use super::{FocusMode, MeasureNode, NodeProperties, dom_node, refresh_dom};
use crate::events::{Event, EventHandlers, queue_event};
use crate::{AsDomNode, Borders, DomNode, NodeType, push_accesskit_tree_update};

new_key_type! { pub struct DomNodeKey; }

thread_local! {
    static DOM_NODES: RefCell<NodeTree> = RefCell::new(NodeTree::new());
}

pub fn with_nodes<F, R>(f: F) -> R
where
    F: FnOnce(&NodeTree) -> R,
{
    DOM_NODES.with(|n| f(&n.borrow()))
}

pub fn with_nodes_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut NodeTree) -> R,
{
    DOM_NODES.with(|n| f(&mut n.borrow_mut()))
}

pub(crate) fn tree_is_accessible() -> bool {
    DOM_NODES.try_with(|_| {}).is_ok()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MatchBehavior {
    StopOnFistMatch,
    ContinueOnMatch,
    SearchChildrenOnMatch,
}

#[derive(Debug)]
pub(crate) struct ContentRect {
    pub(crate) size: Size<u16>,
    content_box_size: Size<u16>,
    pub(crate) content_size: Size<u16>,
    border: taffy::Rect<u16>,
    padding: taffy::Rect<u16>,
    viewport_size: ViewportSize,
    scroll_offset: Position,
    max_scroll_offset: Position,
    x: u16,
    y: u16,
}

impl ContentRect {
    pub(crate) fn render_bounds(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.size.width,
            height: self.size.height,
        }
    }

    pub(crate) fn visible_bounds(&self) -> Rect {
        let viewport = self.viewport_size.viewport();
        Rect {
            x: self.x,
            y: self.y,
            width: self.size.width.min(viewport.width),
            height: self.size.height.min(viewport.height),
        }
    }

    pub(crate) fn scroll_offset(&self) -> Position {
        self.scroll_offset
    }

    pub(crate) fn inner_bounds(&self) -> Rect {
        let width = self.content_size.width.max(self.size.width);
        let height = self.content_size.height.max(self.size.height);
        Rect {
            x: self.x + self.border.left + self.padding.left,
            y: self.y + self.border.top + self.padding.top,
            width: width
                - self.padding.left
                - self.padding.right
                - self.border.left
                - self.border.right,
            height: height
                - self.padding.top
                - self.padding.bottom
                - self.border.top
                - self.border.bottom,
        }
    }

    pub(crate) fn child_bounds(&self) -> Rect {
        Rect {
            x: self.x + self.border.left + self.padding.left,
            y: self.y + self.border.top + self.padding.top,
            width: self.content_box_size.width,
            height: self.content_box_size.height,
        }
    }

    pub(crate) fn can_scroll(&self) -> bool {
        self.max_scroll_offset != Position::ORIGIN
    }

    pub(crate) fn total_size(&self) -> Rect {
        Rect {
            width: self.size.width.max(self.content_size.width),
            height: self.size.height.max(self.content_size.height),
            x: self.x,
            y: self.y,
        }
    }

    pub(crate) fn resize_for_render(&self, buf: &mut Buffer) {
        let mut new = buf.area;
        new.width = new.width.max(self.content_size.width).max(self.size.width);
        new.height = new
            .height
            .max(self.content_size.height)
            .max(self.size.height);

        buf.resize(new);
    }

    pub(crate) fn apply_scroll(&self, rect: Rect, buf: &mut Buffer) {
        if self.scroll_offset == Position::ORIGIN {
            return;
        }
        for row in rect.rows().skip(self.scroll_offset.y as usize) {
            for col in row.columns() {
                let pos: Position = col.into();
                let mut new_pos = pos;
                new_pos.y -= self.scroll_offset.y;
                if new_pos.x < buf.area.x + buf.area.width
                    && new_pos.y < buf.area.y + buf.area.height
                {
                    buf[new_pos] = buf[pos].clone();
                }
            }
        }
    }
}

impl DomNodeKey {
    pub(crate) fn traverse<F, T>(&self, mut f: F, match_behavior: MatchBehavior) -> Vec<T>
    where
        F: FnMut(DomNodeKey, &NodeProperties) -> Option<T>,
    {
        let mut out_list = vec![];
        self.traverse_inner(&mut f, &mut out_list, match_behavior);
        out_list
    }

    fn traverse_inner<F, T>(&self, f: &mut F, out_list: &mut Vec<T>, match_behavior: MatchBehavior)
    where
        F: FnMut(DomNodeKey, &NodeProperties) -> Option<T>,
    {
        if let Some(out) = with_nodes(|nodes| f(*self, &nodes[*self])) {
            out_list.push(out);
            if match_behavior == MatchBehavior::StopOnFistMatch {
                return;
            }
        }
        let children = with_nodes(|nodes| nodes[*self].children.clone());
        for child in children {
            let current_length = out_list.len();
            child.traverse_inner(f, out_list, match_behavior);
            if out_list.len() > current_length
                && match_behavior == MatchBehavior::SearchChildrenOnMatch
            {
                return;
            }
        }
    }
}

pub(crate) struct TreeValue {
    pub(crate) inner: NodeProperties,
    layout_id: Option<NodeId>,
}

#[derive(Clone)]
struct Context {
    offset: Point<f32>,
    key: DomNodeKey,
    accesskit_node: accesskit::Node,
    accesskit_node_id: accesskit::NodeId,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            offset: Point::default(),
            key: DomNodeKey::default(),
            accesskit_node: accesskit::Node::default(),
            accesskit_node_id: accesskit::NodeId(0),
        }
    }
}

static ROOT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(PartialEq, Eq)]
struct RootId {
    z_index: i32,
    id: u32,
}

impl RootId {
    fn new(z_index: i32) -> Self {
        Self {
            z_index,
            id: ROOT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Ord for RootId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.z_index.cmp(&other.z_index) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for RootId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewportSize {
    pub(crate) window_size: Rect,
    pub(crate) max_width: Option<u16>,
    pub(crate) max_height: Option<u16>,
}

impl ViewportSize {
    pub fn window_size(&self) -> Rect {
        self.window_size
    }

    pub fn viewport(&self) -> Rect {
        let Rect { width, height, .. } = self.window_size;
        Rect {
            x: self.window_size.x,
            y: self.window_size.y,
            width: width.min(self.max_width.unwrap_or(width)),
            height: height.min(self.max_height.unwrap_or(width)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FocusableNode {
    key: DomNodeKey,
    focus_mode: FocusMode,
}

impl FocusableNode {
    pub fn new(key: DomNodeKey, focus_mode: FocusMode) -> Self {
        Self { key, focus_mode }
    }
}

pub(crate) enum FocusEventType {
    Next,
    NextList,
    Previous,
    PreviousList,
}

const ACCESSKIT_WINDOW_ID: accesskit::NodeId = accesskit::NodeId(0);

pub struct NodeTree {
    dom_nodes: SlotMap<DomNodeKey, TreeValue>,
    layout_tree: TaffyTree<Context>,
    //accesskit_tree: accesskit::Tree,
    roots: BTreeMap<RootId, Box<dyn AsDomNode>>,
    viewport_size: ViewportSize,
    focused: Option<crate::NodeId>,
    focused_key: Option<DomNodeKey>,
    hovered_key: Option<DomNodeKey>,
    focusable_nodes: Rc<RefCell<Vec<FocusableNode>>>,
    on_window_size_change: Box<dyn Fn(ViewportSize)>,
    on_focus_change: Box<dyn Fn(Option<crate::NodeId>)>,
}

impl Index<DomNodeKey> for NodeTree {
    type Output = NodeProperties;

    fn index(&self, index: DomNodeKey) -> &Self::Output {
        &self.dom_nodes[index].inner
    }
}

impl Default for NodeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeTree {
    pub(crate) fn new() -> Self {
        let accesskit_tree = accesskit::Tree::new(ACCESSKIT_WINDOW_ID);
        push_accesskit_tree_update(TreeUpdate {
            nodes: vec![],
            tree: Some(accesskit_tree),
            tree_id: accesskit::TreeId::ROOT,
            focus: ACCESSKIT_WINDOW_ID,
        });
        Self {
            layout_tree: TaffyTree::<Context>::new(),
            //accesskit_tree,
            roots: BTreeMap::default(),
            dom_nodes: Default::default(),
            focused: None,
            focused_key: None,
            hovered_key: None,
            focusable_nodes: Default::default(),
            viewport_size: ViewportSize::default(),
            on_window_size_change: Box::new(move |_| {}),
            on_focus_change: Box::new(move |_| {}),
        }
    }

    pub fn recompute_full_layout(&mut self, rect: Rect) {
        let root_keys: Vec<_> = self
            .roots
            .values()
            .map(|r| r.as_dom_node().get_key())
            .collect();
        for root_key in root_keys {
            let child_keys = if self.dom_nodes[root_key].layout_id.is_none() {
                self.child_layout_keys(root_key)
            } else {
                vec![root_key]
            };
            for child_key in child_keys {
                let node = &self.dom_nodes[child_key];
                self.layout_tree
                    .compute_layout_with_measure(
                        node.layout_id.unwrap(),
                        Size::<AvailableSpace> {
                            width: AvailableSpace::Definite(rect.width.into()),
                            height: AvailableSpace::Definite(rect.height.into()),
                        },
                        |known_dimensions, available_space, _node_id, node_context, style| {
                            if let Some(node_context) = node_context {
                                let node = &self.dom_nodes[node_context.key];
                                if let NodeType::Widget(widget) = &node.inner.node_type {
                                    return widget.measure(
                                        known_dimensions,
                                        available_space,
                                        style,
                                    );
                                }
                            }
                            Size::zero()
                        },
                    )
                    .unwrap();
                // self.layout_tree.print_tree(node.layout_id.unwrap());
                self.recompute_offsets(
                    node.layout_id.unwrap(),
                    Point {
                        // inline viewports will likely have an initial offset relative to the total
                        // size of the terminal
                        x: rect.x as f32,
                        y: rect.y as f32,
                    },
                );
            }
        }
    }

    pub fn force_recompute_layout(&mut self, key: DomNodeKey) {
        self.layout_tree
            .mark_dirty(self.dom_nodes[key].layout_id.unwrap())
            .unwrap();
    }

    pub fn set_root(&mut self, z_index: i32, root: impl AsDomNode + 'static) {
        self.roots.insert(RootId::new(z_index), Box::new(root));
    }

    pub fn on_window_size_change<F>(&mut self, f: F)
    where
        F: Fn(ViewportSize) + 'static,
    {
        self.on_window_size_change = Box::new(f);
    }

    pub fn on_focus_change<F>(&mut self, f: F)
    where
        F: Fn(Option<crate::NodeId>) + 'static,
    {
        self.on_focus_change = Box::new(f);
    }

    pub(crate) fn root(&self, z_index: i32) -> &dyn AsDomNode {
        let key = &self.roots.keys().find(|k| k.z_index == z_index).unwrap();
        &self.roots[key]
    }

    pub fn clear(&mut self) {
        let viewport_size = self.viewport_size;
        *self = Self::new();
        self.viewport_size = viewport_size;
    }

    pub fn roots_asc(&self) -> Vec<DomNode> {
        self.roots
            .values()
            .map(|r| r.as_dom_node().clone())
            .collect()
    }

    pub fn roots_desc(&self) -> Vec<DomNode> {
        let mut roots: Vec<_> = self
            .roots
            .values()
            .map(|r| r.as_dom_node().clone())
            .collect();
        roots.reverse();
        roots
    }

    pub(crate) fn set_unmounted(&self, key: DomNodeKey, unmounted: bool) {
        self.dom_nodes[key]
            .inner
            .unmounted
            .store(unmounted, Ordering::Relaxed);
    }

    pub(crate) fn focused_key(&self) -> Option<DomNodeKey> {
        self.focused_key
    }

    pub fn focused(&self) -> &Option<crate::NodeId> {
        &self.focused
    }

    pub(crate) fn focus_event_type(&self, key_event: &KeyEvent) -> Option<FocusEventType> {
        if key_event.kind == KeyEventKind::Release {
            return None;
        }
        if let Some(focused) = self.focused_key {
            let focusable_nodes = self.focusable_nodes.borrow();
            let focusable = focusable_nodes.iter().find(|n| n.key == focused).unwrap();
            match key_event {
                key!(KeyCode::Tab) if focusable.focus_mode.is_tab_focus() => {
                    Some(FocusEventType::Next)
                }
                key!(SHIFT, KeyCode::Tab) if focusable.focus_mode.is_tab_focus() => {
                    Some(FocusEventType::Previous)
                }
                key!(KeyCode::Down) => {
                    let mut key = focusable.key;
                    while let Some(parent) = self.dom_nodes[key].inner.parent {
                        if self.dom_nodes[parent]
                            .inner
                            .focus_mode
                            .is_vertical_list_focus()
                        {
                            return Some(FocusEventType::NextList);
                        }
                        key = parent;
                    }
                    None
                }
                key!(KeyCode::Up) => {
                    let mut key = focusable.key;
                    while let Some(parent) = self.dom_nodes[key].inner.parent {
                        if self.dom_nodes[parent]
                            .inner
                            .focus_mode
                            .is_vertical_list_focus()
                        {
                            return Some(FocusEventType::PreviousList);
                        }
                        key = parent;
                    }
                    None
                }
                key!(KeyCode::Left) => {
                    let mut key = focusable.key;
                    while let Some(parent) = self.dom_nodes[key].inner.parent {
                        if self.dom_nodes[parent]
                            .inner
                            .focus_mode
                            .is_horizontal_list_focus()
                        {
                            return Some(FocusEventType::PreviousList);
                        }
                        key = parent;
                    }
                    None
                }
                key!(KeyCode::Right) => {
                    let mut key = focusable.key;
                    while let Some(parent) = self.dom_nodes[key].inner.parent {
                        if self.dom_nodes[parent]
                            .inner
                            .focus_mode
                            .is_horizontal_list_focus()
                        {
                            return Some(FocusEventType::NextList);
                        }
                        key = parent;
                    }
                    None
                }
                _ => None,
            }
        } else {
            match key_event {
                key!(KeyCode::Tab) => Some(FocusEventType::Next),
                key!(SHIFT, KeyCode::Tab) => Some(FocusEventType::Previous),
                _ => None,
            }
        }
    }

    pub(crate) fn hovered_key(&self) -> Option<DomNodeKey> {
        self.hovered_key
    }

    pub(crate) fn set_viewport_size(&mut self, viewport_size: ViewportSize) {
        if viewport_size != self.viewport_size {
            self.viewport_size = viewport_size;
            (self.on_window_size_change)(viewport_size);
        }
    }

    pub fn node_type(&self, key: DomNodeKey) -> &NodeType {
        &self.dom_nodes[key].inner.node_type
    }

    pub fn original_display(&self, key: DomNodeKey) -> &taffy::Display {
        &self.dom_nodes[key].inner.original_display
    }

    pub fn viewport_size(&self) -> ViewportSize {
        self.viewport_size
    }

    pub fn clear_focusables(&mut self) {
        self.focusable_nodes.borrow_mut().clear();
    }

    pub(crate) fn child_layout_keys(&self, key: DomNodeKey) -> Vec<DomNodeKey> {
        let mut keys = Vec::new();
        for child in &self.dom_nodes[key].inner.children {
            let child_node = &self.dom_nodes[*child];
            if child_node.layout_id.is_none()
                || child_node.inner.original_display == taffy::Display::None
            {
                for child in &child_node.inner.children {
                    keys.append(&mut self.child_layout_keys(*child));
                }
            } else {
                keys.push(*child);
            }
        }
        keys
    }

    fn recompute_offsets(&mut self, root_layout: NodeId, root_offset: Point<f32>) {
        let root_context = self.layout_tree.get_node_context_mut(root_layout).unwrap();
        root_context.offset.x = root_offset.x;
        root_context.offset.y = root_offset.y;

        self.recompute_offset(root_layout);
    }

    fn recompute_offset(&mut self, layout_node: NodeId) {
        let context = self.layout_tree.get_node_context(layout_node).unwrap();
        let key = context.key;
        let layout = self.layout_tree.layout(layout_node).unwrap();

        let scroll_height =
            (layout.scroll_height() as u16).saturating_sub(layout.scrollbar_size.height as u16);
        let scroll_width =
            (layout.scroll_width() as u16).saturating_sub(layout.scrollbar_size.width as u16);

        let viewport = self.viewport_size.viewport();
        if (layout.size.height as u16 > viewport.height
            && self.style(key).overflow.y == Overflow::Scroll)
            || (layout.size.width as u16 > viewport.width
                && self.style(key).overflow.x == Overflow::Scroll)
        {
            let max_x = (layout.size.width as u16).saturating_sub(viewport.width);
            let max_y = (layout.size.height as u16).saturating_sub(viewport.height);
            self.dom_nodes[key].inner.max_scroll_offset = Position::new(max_x, max_y);
            let scroll_offset = self.dom_nodes[key].inner.scroll_offset;
            self.dom_nodes[key].inner.scroll_offset.x = scroll_offset.x.min(max_x);
            self.dom_nodes[key].inner.scroll_offset.y = scroll_offset.y.min(max_y);
        } else if (scroll_height > 0 && self.style(key).overflow.y == Overflow::Scroll)
            || (scroll_width > 0 && self.style(key).overflow.x == Overflow::Scroll)
        {
            self.dom_nodes[key].inner.max_scroll_offset =
                Position::new(scroll_width, scroll_height);
            let scroll_offset = self.dom_nodes[key].inner.scroll_offset;
            self.dom_nodes[key].inner.scroll_offset.x = scroll_offset.x.min(scroll_width);
            self.dom_nodes[key].inner.scroll_offset.y = scroll_offset.y.min(scroll_height);
        } else {
            self.dom_nodes[key].inner.max_scroll_offset = Position::default();
            self.dom_nodes[key].inner.scroll_offset = Position::default();
        }
        let context = self
            .layout_tree
            .get_node_context(layout_node)
            .unwrap()
            .offset;
        for child_node in self.layout_tree.children(layout_node).unwrap() {
            self.compute_location(child_node, &context);
            self.recompute_offset(child_node);
        }
    }

    fn compute_location(&mut self, child: NodeId, parent_offset: &Point<f32>) {
        let child_layout = *self.layout_tree.layout(child).unwrap();
        let context = self.layout_tree.get_node_context_mut(child).unwrap();

        let new_x = child_layout.location.x + parent_offset.x;
        let new_y = child_layout.location.y + parent_offset.y;
        // if context.0.x != new_x || context.0.y != new_y {
        context.offset.x = new_x;
        context.offset.y = new_y;
        let width = child_layout.content_size.width as f64;
        let height = child_layout.content_size.height as f64;
        context.accesskit_node.set_bounds(accesskit::Rect {
            x0: new_x as f64,
            y0: new_y as f64,
            x1: new_x as f64 + width,
            y1: new_y as f64 + height,
        });
    }

    pub fn update_layout<F>(&mut self, node: DomNodeKey, mut f: F)
    where
        F: FnMut(&mut Style),
    {
        let value = &self.dom_nodes[node];
        let mut style = self
            .layout_tree
            .style(value.layout_id.unwrap())
            .unwrap()
            .clone();

        f(&mut style);

        self.layout_tree
            .set_style(value.layout_id.unwrap(), style.clone())
            .unwrap();

        refresh_dom();
    }

    pub(crate) fn rect(&self, key: DomNodeKey) -> ContentRect {
        self.try_rect(key).unwrap()
    }

    pub(crate) fn get_accesskit_node_id(&self, key: NodeId) -> accesskit::NodeId {
        self.layout_tree
            .get_node_context(key)
            .unwrap()
            .accesskit_node_id
    }

    pub(crate) fn try_rect(&self, key: DomNodeKey) -> Option<ContentRect> {
        let layout_id = self.dom_nodes[key].layout_id?;
        let context = self.layout_tree.get_node_context(layout_id).unwrap();
        let computed = self.get_computed(key);
        let scroll_offset = self.dom_nodes[key].inner.scroll_offset;
        let max_scroll_offset = self.dom_nodes[key].inner.max_scroll_offset;

        Some(ContentRect {
            x: context.offset.x as u16,
            y: context.offset.y as u16,
            content_box_size: computed.content_box_size().map(|s| s as u16),
            size: computed.size.map(|s| s as u16),
            viewport_size: self.viewport_size,
            content_size: computed.content_size.map(|s| s as u16),
            border: computed.border.map(|s| s as u16),
            padding: computed.padding.map(|s| s as u16),
            scroll_offset,
            max_scroll_offset,
        })
    }

    fn get_computed(&self, key: DomNodeKey) -> &taffy::Layout {
        self.layout_tree
            .layout(self.dom_nodes[key].layout_id.unwrap())
            .unwrap()
    }

    pub(crate) fn try_style(&self, key: DomNodeKey) -> Option<&taffy::Style> {
        if let Some(layout_id) = self.dom_nodes[key].layout_id {
            Some(self.layout_tree.style(layout_id).unwrap())
        } else {
            None
        }
    }

    pub(crate) fn style(&self, key: DomNodeKey) -> &taffy::Style {
        self.try_style(key).unwrap()
    }

    pub(crate) fn iter_nodes(&self) -> slotmap::basic::Iter<'_, DomNodeKey, TreeValue> {
        self.dom_nodes.iter()
    }

    pub(crate) fn iter_layout_nodes(
        &self,
    ) -> impl Iterator<Item = (DomNodeKey, taffy::NodeId, &TreeValue)> {
        self.dom_nodes
            .iter()
            .filter_map(|(k, v)| v.layout_id.map(|i| (k, i, v)))
    }

    pub(crate) fn contains_key(&self, key: DomNodeKey) -> bool {
        self.dom_nodes.contains_key(key)
    }

    pub(crate) fn insert(&mut self, val: NodeProperties) -> DomNodeKey {
        if matches!(val.node_type, NodeType::FocusScope(_)) {
            let key = self.dom_nodes.insert(TreeValue {
                inner: val,
                layout_id: None,
            });
            return key;
        }
        let style = Style {
            scrollbar_width: 1.0,
            // overflow: Point {
            //     x: Overflow::Scroll,
            //     y: Overflow::Scroll,
            // },
            ..Default::default()
        };
        let layout_node = self
            .layout_tree
            .new_leaf_with_context(style, Context::default())
            .unwrap();
        let accesskit_role = match &val.node_type {
            // handled above
            NodeType::FocusScope(_) => unreachable!(),
            NodeType::Layout | NodeType::Placeholder => accesskit::Role::GenericContainer,
            NodeType::Widget(widget) => widget
                .role
                .map(|r| r.into())
                .unwrap_or(accesskit::Role::GenericContainer),
        };
        let key = self.dom_nodes.insert(TreeValue {
            inner: val,
            layout_id: Some(layout_node),
        });
        if let NodeType::Widget(widget) = &mut self.dom_nodes[key].inner.node_type {
            widget.set_key(key);
        }
        let context = self.layout_tree.get_node_context_mut(layout_node).unwrap();
        context.key = key;
        context.accesskit_node = accesskit::Node::new(accesskit_role);
        context
            .accesskit_node
            .set_tree_id(accesskit::TreeId(accesskit::Uuid::new_v4()));
        context.accesskit_node_id = accesskit::NodeId(layout_node.into());
        key
    }

    pub(crate) fn insert_before(
        &mut self,
        parent_key: DomNodeKey,
        child_key: DomNodeKey,
        reference: Option<DomNodeKey>,
    ) {
        let parent = &self.dom_nodes[parent_key];
        let child = &self.dom_nodes[child_key];
        if child.inner.z_index.is_some() && parent.inner.z_index != child.inner.z_index {
            return;
        }
        let parent_layout_id = parent.layout_id;
        if let Some(reference) = reference {
            if let Some(reference_pos) = self.dom_nodes[parent_key]
                .inner
                .children
                .iter()
                .position(|c| *c == reference)
            {
                self.dom_nodes[parent_key]
                    .inner
                    .children
                    .insert(reference_pos, child_key);
                self.dom_nodes[child_key].inner.parent = Some(parent_key);
                let layout_parent_key = self.nearest_layout_parent(parent_key);
                if let Some(layout_parent_key) = layout_parent_key {
                    let parent_node = &self.dom_nodes[layout_parent_key];
                    let child_nodes = self.nearest_layout_children(child_key);
                    // Insert in reverse to preserve index ordering
                    for child_key in child_nodes.iter().rev() {
                        let child_node = &self.dom_nodes[*child_key];
                        if layout_parent_key == parent_key {
                            self.layout_tree
                                .insert_child_at_index(
                                    parent_node.layout_id.unwrap(),
                                    reference_pos,
                                    child_node.layout_id.unwrap(),
                                )
                                .unwrap();
                        } else {
                            self.layout_tree
                                .add_child(
                                    parent_node.layout_id.unwrap(),
                                    child_node.layout_id.unwrap(),
                                )
                                .unwrap();
                        }
                    }
                }
            }
        } else {
            self.dom_nodes[parent_key].inner.children.push(child_key);
            self.dom_nodes[child_key].inner.parent = Some(parent_key);
            let layout_parent_key = self.nearest_layout_parent(parent_key);
            if let Some(layout_parent_key) = layout_parent_key {
                let child_nodes = self.nearest_layout_children(child_key);
                for child_key in child_nodes {
                    let parent_node = &self.dom_nodes[layout_parent_key];
                    let child_node = &self.dom_nodes[child_key];
                    self.layout_tree
                        .add_child(
                            parent_node.layout_id.unwrap(),
                            child_node.layout_id.unwrap(),
                        )
                        .unwrap();
                }
            }
        }
        let parent_node = &self.dom_nodes[parent_key];
        // If the parent is disabled, the child should also be disabled
        if !parent_node.inner.enabled() {
            self.set_parent_enabled(false, child_key);
        }
        self.set_unmounted(child_key, false);
        let child_nodes = self.nearest_layout_children(child_key);

        let mut updates = Vec::new();
        let mut children = Vec::new();
        for child_node in child_nodes
            .into_iter()
            .filter_map(|c| self.dom_nodes[c].layout_id)
        {
            let context = self.layout_tree.get_node_context(child_node).unwrap();
            updates.push((context.accesskit_node_id, context.accesskit_node.clone()));
            children.push(context.accesskit_node_id);
        }
        let focused = self
            .focused_key
            .map(|k| {
                let layout_id = self.dom_nodes[k].layout_id.unwrap();
                self.layout_tree
                    .get_node_context(layout_id)
                    .unwrap()
                    .accesskit_node_id
            })
            .unwrap_or(ACCESSKIT_WINDOW_ID);
        if let Some(parent_layout_id) = parent_layout_id {
            let parent_context = self
                .layout_tree
                .get_node_context_mut(parent_layout_id)
                .unwrap();
            parent_context.accesskit_node.set_children(children);
            updates.push((
                parent_context.accesskit_node_id,
                parent_context.accesskit_node.clone(),
            ));
            push_accesskit_tree_update(TreeUpdate {
                nodes: updates,
                tree: None,
                tree_id: parent_context.accesskit_node.tree_id().unwrap(),
                focus: focused,
            });
        } else {
            push_accesskit_tree_update(TreeUpdate {
                nodes: updates,
                tree: None,
                tree_id: accesskit::TreeId::ROOT,
                focus: focused,
            });
        }

        refresh_dom();
    }

    fn nearest_layout_parent(&self, key: DomNodeKey) -> Option<DomNodeKey> {
        if self.dom_nodes[key].layout_id.is_none() {
            let parent = self.dom_nodes[key].inner.parent?;
            return self.nearest_layout_parent(parent);
        }
        Some(key)
    }

    fn nearest_layout_children(&self, key: DomNodeKey) -> Vec<DomNodeKey> {
        if self.dom_nodes[key].layout_id.is_none() {
            self.child_layout_keys(key)
        } else {
            vec![key]
        }
    }

    pub(crate) fn remove(&mut self, node: DomNodeKey) -> Option<TreeValue> {
        if let Some(layout_id) = self.dom_nodes[node].layout_id {
            self.layout_tree.remove(layout_id).unwrap();
        }

        let removed = self.dom_nodes.remove(node);
        refresh_dom();
        removed
    }

    pub(crate) fn unmount_child(&mut self, child: DomNodeKey) {
        let child_node = &self.dom_nodes[child];
        if let Some(parent) = child_node.inner.parent {
            let child_pos = self.dom_nodes[parent]
                .inner
                .children
                .iter()
                .position(|c| c == &child)
                .unwrap();
            self.dom_nodes[parent].inner.children.remove(child_pos);
            self.dom_nodes[child].inner.parent = None;

            let child_node = &self.dom_nodes[child];
            if let Some(layout_id) = child_node.layout_id {
                let parent_layout = self.nearest_layout_parent(parent).unwrap();
                let parent_id = self.dom_nodes[parent_layout].layout_id.unwrap();
                self.layout_tree.remove_child(parent_id, layout_id).unwrap();
            }

            refresh_dom();
        }
    }

    pub(crate) fn replace_node(&mut self, old_key: DomNodeKey, new_key: DomNodeKey) {
        let mut new_key_vals = mem::take(&mut self.dom_nodes[new_key].inner);
        new_key_vals.replace_with(&self.dom_nodes[old_key].inner, new_key);

        self.dom_nodes[new_key].inner = new_key_vals;
        if let Some(parent) = self.dom_nodes[new_key].inner.parent {
            let parent_enabled = self.dom_nodes[parent].inner.enabled();
            self.dom_nodes[new_key]
                .inner
                .set_parent_enabled(parent_enabled);
        }

        refresh_dom();
    }

    pub(crate) fn replace_inner(&mut self, key: DomNodeKey, mut inner: NodeProperties) {
        if let NodeType::Widget(widget) = &mut inner.node_type {
            widget.set_key(key);
        }
        self.dom_nodes[key].inner = inner;

        refresh_dom();
    }

    pub fn set_focus_mode(&mut self, node: DomNodeKey, focus_mode: FocusMode) {
        self.dom_nodes[node].inner.set_focus_mode(focus_mode);
        refresh_dom();
    }

    pub(crate) fn scroll(&mut self, node: DomNodeKey, direction: ScrollDirection) {
        let change = self.dom_nodes[node].inner.scroll(direction);
        self.update_ancestor_scroll_offsets(node, change);
        refresh_dom();
    }

    fn update_ancestor_scroll_offsets(&mut self, node: DomNodeKey, change: taffy::Point<i32>) {
        for child in self.dom_nodes[node].inner.children.clone() {
            self.dom_nodes[child]
                .inner
                .update_ancestor_scroll_offsets(change);
            self.update_ancestor_scroll_offsets(child, change);
        }
    }

    pub fn set_enabled(&mut self, key: DomNodeKey, enabled: bool) {
        self.dom_nodes[key].inner.set_enabled(enabled);

        if !enabled {
            self.unset_state(&key);
        }
        let children = self.dom_nodes[key].inner.children.clone();
        for child in children {
            self.set_parent_enabled(enabled, child);
        }

        if enabled {
            queue_event(Event::NodeEnable(key));
        } else {
            queue_event(Event::NodeDisable(key));
        }

        refresh_dom();
    }

    fn set_parent_enabled(&mut self, enabled: bool, node: DomNodeKey) {
        if !enabled {
            self.unset_state(&node);
        }
        self.dom_nodes[node].inner.set_parent_enabled(enabled);
        let children = self.dom_nodes[node].inner.children.clone();
        for child in children {
            self.set_parent_enabled(enabled, child);
        }
    }

    pub(crate) fn unset_state(&mut self, key: &DomNodeKey) {
        if self.focused_key == Some(*key) {
            self.remove_focused();
        }
        if self.hovered_key == Some(*key) {
            self.remove_hovered();
        }
        self.remove_focusable(key);
    }

    fn remove_focusable(&mut self, key: &DomNodeKey) {
        let mut focusable_nodes = self.focusable_nodes.borrow_mut();
        if let Some(pos) = focusable_nodes.iter().position(|n| n.key == *key) {
            focusable_nodes.remove(pos);
        }
    }

    pub(crate) fn update_event_handlers<F>(&mut self, node: DomNodeKey, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        self.dom_nodes[node].inner.event_handlers =
            update(self.dom_nodes[node].inner.event_handlers.clone())
    }

    pub fn set_id(&mut self, node: DomNodeKey, id: impl Into<dom_node::NodeId>) {
        self.dom_nodes[node].inner.id = Some(id.into());
        refresh_dom();
    }

    pub fn set_class(&mut self, node: DomNodeKey, class: impl Into<Vec<String>>) {
        self.dom_nodes[node].inner.class = class.into();
        refresh_dom();
    }

    pub fn set_borders(&mut self, node: DomNodeKey, borders: Borders) {
        self.dom_nodes[node].inner.borders = Some(borders);
        refresh_dom();
    }

    pub fn set_background(&mut self, node: DomNodeKey, background: Color) {
        self.dom_nodes[node].inner.background = Some(background);
        refresh_dom();
    }

    #[cfg(feature = "effects")]
    pub fn set_effect(&mut self, node: DomNodeKey, effect: tachyonfx::Effect) {
        self.dom_nodes[node].inner.effects =
            Some(RefCell::new(super::EffectProperties::new(effect)));
        refresh_dom();
    }

    pub fn set_z_index(&mut self, key: DomNodeKey, z_index: i32) {
        self.unmount_child(key);
        self.dom_nodes[key].inner.z_index = Some(z_index);

        let unmounted = self.dom_nodes[key].inner.unmounted.clone();
        let node = DomNode::from_existing(key, unmounted);
        self.roots.insert(RootId::new(z_index), Box::new(node));
        refresh_dom();
    }

    pub fn set_clear(&mut self, key: DomNodeKey, clear: bool) {
        self.dom_nodes[key].inner.clear = clear;
        refresh_dom();
    }

    pub(crate) fn add_focusable(&self, key: DomNodeKey, focus_mode: FocusMode) {
        if !self.dom_nodes[key].inner.enabled() {
            return;
        }

        self.focusable_nodes
            .borrow_mut()
            .push(FocusableNode { key, focus_mode });
    }

    pub(crate) fn remove_hovered(&mut self) {
        self.hovered_key = None;
    }

    fn remove_focused(&mut self) {
        if let Some(focused_key) = self.focused_key {
            let node_id = self.dom_nodes[focused_key].inner.id.clone();

            queue_event(Event::NodeBlur {
                blur_key: focused_key,
                focus_target: node_id,
            });
        }
        self.set_focused(None);
        self.focused_key = None;
    }

    pub fn get_key(&self, id: impl Into<crate::NodeId>) -> Option<DomNodeKey> {
        let id = id.into();
        self.dom_nodes
            .iter()
            .find(|(_, v)| v.inner.id.as_ref() == Some(&id))
            .map(|(k, _)| k)
    }

    pub fn set_focused_untracked(&mut self, key: Option<DomNodeKey>) {
        let Some(key) = key else {
            self.focused = None;
            self.focused_key = None;
            return;
        };
        if !self.dom_nodes.contains_key(key) {
            self.focused = None;
            self.focused_key = None;
            return;
        }
        self.focused_key = Some(key);
        self.focused = self.dom_nodes[key].inner.id.clone();
    }

    pub fn set_focused(&mut self, node: Option<DomNodeKey>) {
        let prev_focused_id = if let Some(focused_key) = self.focused_key {
            let node_id = self.dom_nodes[focused_key].inner.id.clone();

            queue_event(Event::NodeBlur {
                blur_key: focused_key,
                focus_target: node_id.clone(),
            });
            node_id
        } else {
            None
        };

        self.set_focused_untracked(node);
        if prev_focused_id != self.focused {
            (self.on_focus_change)(self.focused.clone());
        }
        let Some(node_key) = self.focused_key else {
            refresh_dom();
            return;
        };

        queue_event(Event::NodeFocus {
            focus_key: node_key,
            prev_focused: prev_focused_id,
        });
        refresh_dom();
    }

    pub(crate) fn set_hovered(&mut self, node_key: DomNodeKey) {
        self.hovered_key = Some(node_key);
    }

    pub(crate) fn focus_next(&mut self) {
        if let Some(focused) = self.focused_key {
            let current_focused = self
                .focusable_nodes
                .borrow()
                .iter()
                .position(|n| n.key == focused)
                .unwrap();

            let last = self.focusable_nodes.borrow().len() - 1;
            if current_focused < last {
                let next = self.focusable_nodes.borrow()[current_focused + 1];
                self.set_focused(Some(next.key));
                return;
            }
        }
        let next = self.focusable_nodes.borrow().first().cloned();
        if let Some(next) = next {
            self.set_focused(Some(next.key));
        }
    }

    pub(crate) fn focus_next_list(&mut self) {
        if let Some(focused) = self.focused_key {
            let mut key = focused;
            let mut child_index = None;
            let mut child_layout_keys = Vec::new();
            while let Some(parent) = self.dom_nodes[key].inner.parent {
                let parent_node = &self.dom_nodes[parent];
                if parent_node.inner.list_focusable() {
                    child_layout_keys = self.child_layout_keys(parent);
                    child_index = child_layout_keys.iter().position(|c| *c == key);
                    break;
                }
                key = parent;
            }
            if let Some(child_index) = child_index {
                let next = child_layout_keys[child_index + 1..]
                    .iter()
                    .find(|k| self.dom_nodes[**k].inner.focusable());
                if let Some(next) = next {
                    self.set_focused(Some(*next));
                    return;
                }

                let first = child_layout_keys
                    .iter()
                    .find(|k| self.dom_nodes[**k].inner.focusable());
                self.set_focused(first.copied());
            }
        }
    }

    pub(crate) fn focus_prev(&mut self) {
        if let Some(focused) = self.focused_key {
            let current_focused = self
                .focusable_nodes
                .borrow()
                .iter()
                .position(|n| n.key == focused)
                .unwrap();
            if current_focused > 0 {
                let prev = self.focusable_nodes.borrow()[current_focused - 1];
                self.set_focused(Some(prev.key));
                return;
            }
        }
        let prev = self.focusable_nodes.borrow().last().cloned();
        if let Some(prev) = prev {
            self.set_focused(Some(prev.key));
        }
    }

    pub(crate) fn focus_prev_list(&mut self) {
        if let Some(focused) = self.focused_key {
            let mut key = focused;
            let mut child_index = None;
            let mut child_layout_keys = Vec::new();
            while let Some(parent) = self.dom_nodes[key].inner.parent {
                let parent_node = &self.dom_nodes[parent];
                if parent_node.inner.list_focusable() {
                    child_layout_keys = self.child_layout_keys(parent);
                    child_index = child_layout_keys.iter().position(|c| *c == key);
                    break;
                }
                key = parent;
            }
            if let Some(child_index) = child_index {
                let prev = child_layout_keys[0..child_index]
                    .iter()
                    .rev()
                    .find(|k| self.dom_nodes[**k].inner.focusable());
                if let Some(next) = prev {
                    self.set_focused(Some(*next));
                    return;
                }

                let last = child_layout_keys
                    .iter()
                    .rev()
                    .find(|k| self.dom_nodes[**k].inner.focusable());
                self.set_focused(last.copied());
            }
        }
    }
}

pub fn focus_next() {
    with_nodes_mut(|n| n.focus_next())
}

pub fn focus_next_list() {
    with_nodes_mut(|n| n.focus_next_list())
}

pub fn focus_prev() {
    with_nodes_mut(|n| n.focus_prev())
}

pub fn focus_prev_list() {
    with_nodes_mut(|n| n.focus_prev_list())
}

pub fn clear_focus() {
    with_nodes_mut(|n| n.set_focused(None))
}
