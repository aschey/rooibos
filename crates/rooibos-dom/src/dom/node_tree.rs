use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{cmp, mem};

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::Block;
use slotmap::{SlotMap, new_key_type};
use taffy::{AvailableSpace, NodeId, Overflow, Point, Size, Style, TaffyTree};
use terminput::ScrollDirection;

use super::{MeasureNode, NodeProperties, dom_node, refresh_dom};
use crate::events::{BlurEvent, Event, EventData, EventHandlers, FocusEvent, queue_event};
use crate::{AsDomNode, DomNode, NodeType};

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
    layout_id: NodeId,
}

#[derive(Clone, Default)]
struct Context {
    offset: Point<f32>,
    key: DomNodeKey,
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

pub struct NodeTree {
    dom_nodes: SlotMap<DomNodeKey, TreeValue>,
    layout_tree: TaffyTree<Context>,
    roots: BTreeMap<RootId, Box<dyn AsDomNode>>,
    viewport_size: ViewportSize,
    focused: Option<crate::NodeId>,
    focused_key: Option<DomNodeKey>,
    hovered_key: Option<DomNodeKey>,
    focusable_nodes: Rc<RefCell<Vec<DomNodeKey>>>,
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
        Self {
            layout_tree: TaffyTree::<Context>::new(),
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
            let node = &self.dom_nodes[root_key];
            self.layout_tree
                .compute_layout_with_measure(
                    node.layout_id,
                    Size::<AvailableSpace> {
                        width: AvailableSpace::Definite(rect.width.into()),
                        height: AvailableSpace::Definite(rect.height.into()),
                    },
                    |known_dimensions, available_space, _node_id, node_context, style| {
                        if let Some(node_context) = node_context {
                            let node = &self.dom_nodes[node_context.key];
                            if let NodeType::Widget(widget) = &node.inner.node_type {
                                return widget.measure(known_dimensions, available_space, style);
                            }
                        }
                        Size::zero()
                    },
                )
                .unwrap();
            self.recompute_offsets(
                root_key,
                Point {
                    // inline viewports will likely have an initial offset relative to the total
                    // size of the terminal
                    x: rect.x as f32,
                    y: rect.y as f32,
                },
            );
        }
    }

    pub fn force_recompute_layout(&mut self, key: DomNodeKey) {
        self.layout_tree
            .mark_dirty(self.dom_nodes[key].layout_id)
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

    fn recompute_offsets(&mut self, root_key: DomNodeKey, root_offset: Point<f32>) {
        let root = self.dom_nodes[root_key].layout_id;
        let root_context = self.layout_tree.get_node_context_mut(root).unwrap();
        root_context.offset.x = root_offset.x;
        root_context.offset.y = root_offset.y;
        self.recompute_offset(root_key);
    }

    fn recompute_offset(&mut self, parent_key: DomNodeKey) {
        let parent = self.dom_nodes[parent_key].layout_id;
        let parent_context = self.layout_tree.get_node_context(parent).unwrap().clone();
        let children = self.dom_nodes[parent_key].inner.children.clone();
        let layout = self.layout_tree.layout(parent).unwrap();

        let scroll_height =
            (layout.scroll_height() as u16).saturating_sub(layout.scrollbar_size.height as u16);
        let scroll_width =
            (layout.scroll_width() as u16).saturating_sub(layout.scrollbar_size.width as u16);

        let viewport = self.viewport_size.viewport();
        if (layout.size.height as u16 > viewport.height
            && self.style(parent_key).overflow.y == Overflow::Scroll)
            || (layout.size.width as u16 > viewport.width
                && self.style(parent_key).overflow.x == Overflow::Scroll)
        {
            let max_x = (layout.size.width as u16).saturating_sub(viewport.width);
            let max_y = (layout.size.height as u16).saturating_sub(viewport.height);
            self.dom_nodes[parent_key].inner.max_scroll_offset = Position::new(max_x, max_y);
            let scroll_offset = self.dom_nodes[parent_key].inner.scroll_offset;
            self.dom_nodes[parent_key].inner.scroll_offset.x = scroll_offset.x.min(max_x);
            self.dom_nodes[parent_key].inner.scroll_offset.y = scroll_offset.y.min(max_y);
        } else if (scroll_height > 0 && self.style(parent_key).overflow.y == Overflow::Scroll)
            || (scroll_width > 0 && self.style(parent_key).overflow.x == Overflow::Scroll)
        {
            self.dom_nodes[parent_key].inner.max_scroll_offset =
                Position::new(scroll_width, scroll_height);
            let scroll_offset = self.dom_nodes[parent_key].inner.scroll_offset;
            self.dom_nodes[parent_key].inner.scroll_offset.x = scroll_offset.x.min(scroll_width);
            self.dom_nodes[parent_key].inner.scroll_offset.y = scroll_offset.y.min(scroll_height);
        } else {
            self.dom_nodes[parent_key].inner.max_scroll_offset = Position::default();
            self.dom_nodes[parent_key].inner.scroll_offset = Position::default();
        }

        for child_key in children {
            let child = self.dom_nodes[child_key].layout_id;
            let child_layout = *self.layout_tree.layout(child).unwrap();
            let context = self.layout_tree.get_node_context_mut(child).unwrap();
            let new_x = child_layout.location.x + parent_context.offset.x;
            let new_y = child_layout.location.y + parent_context.offset.y;
            // if context.0.x != new_x || context.0.y != new_y {
            context.offset.x = new_x;
            context.offset.y = new_y;
            self.recompute_offset(child_key);
            // }
        }
    }

    pub fn update_layout<F>(&mut self, node: DomNodeKey, mut f: F)
    where
        F: FnMut(&mut Style),
    {
        let value = &self.dom_nodes[node];
        let mut style = self.layout_tree.style(value.layout_id).unwrap().clone();

        f(&mut style);

        self.layout_tree
            .set_style(value.layout_id, style.clone())
            .unwrap();

        refresh_dom();
    }

    pub(crate) fn rect(&self, key: DomNodeKey) -> ContentRect {
        let context = self
            .layout_tree
            .get_node_context(self.dom_nodes[key].layout_id)
            .unwrap();
        let computed = self.get_computed(key);
        let scroll_offset = self.dom_nodes[key].inner.scroll_offset;
        let max_scroll_offset = self.dom_nodes[key].inner.max_scroll_offset;

        ContentRect {
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
        }
    }

    fn get_computed(&self, key: DomNodeKey) -> &taffy::Layout {
        self.layout_tree
            .layout(self.dom_nodes[key].layout_id)
            .unwrap()
    }

    pub(crate) fn style(&self, key: DomNodeKey) -> &taffy::Style {
        self.layout_tree
            .style(self.dom_nodes[key].layout_id)
            .unwrap()
    }

    pub(crate) fn iter_nodes(&self) -> slotmap::basic::Iter<'_, DomNodeKey, TreeValue> {
        self.dom_nodes.iter()
    }

    pub(crate) fn contains_key(&self, key: DomNodeKey) -> bool {
        self.dom_nodes.contains_key(key)
    }

    pub(crate) fn insert(&mut self, val: NodeProperties) -> DomNodeKey {
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
        let key = self.dom_nodes.insert(TreeValue {
            inner: val,
            layout_id: layout_node,
        });
        if let NodeType::Widget(widget) = &mut self.dom_nodes[key].inner.node_type {
            widget.set_key(key);
        }
        self.layout_tree
            .get_node_context_mut(layout_node)
            .unwrap()
            .key = key;
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
                let parent_node = &self.dom_nodes[parent_key];
                let child_node = &self.dom_nodes[child_key];
                self.layout_tree
                    .insert_child_at_index(
                        parent_node.layout_id,
                        reference_pos,
                        child_node.layout_id,
                    )
                    .unwrap();
            }
        } else {
            self.dom_nodes[parent_key].inner.children.push(child_key);
            self.dom_nodes[child_key].inner.parent = Some(parent_key);
            let parent_node = &self.dom_nodes[parent_key];
            let child_node = &self.dom_nodes[child_key];
            self.layout_tree
                .add_child(parent_node.layout_id, child_node.layout_id)
                .unwrap();
        }
        let parent_node = &self.dom_nodes[parent_key];
        // If the parent is disabled, the child should also be disabled
        if !parent_node.inner.enabled() {
            self.set_parent_enabled(false, child_key);
        }
        self.set_unmounted(child_key, false);
        refresh_dom();
    }

    pub(crate) fn remove(&mut self, node: DomNodeKey) -> Option<TreeValue> {
        let layout_id = self.dom_nodes[node].layout_id;
        self.layout_tree.remove(layout_id).unwrap();
        refresh_dom();
        self.dom_nodes.remove(node)
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
            let parent_node = &self.dom_nodes[parent];
            let child_node = &self.dom_nodes[child];
            self.layout_tree
                .remove_child(parent_node.layout_id, child_node.layout_id)
                .unwrap();
            refresh_dom();
        }
    }

    pub(crate) fn replace_node(&mut self, old_key: DomNodeKey, new_key: DomNodeKey) {
        let mut new_key_vals = mem::take(&mut self.dom_nodes[new_key].inner);
        new_key_vals.replace_with(&self.dom_nodes[old_key].inner);
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

    pub fn set_focusable(&mut self, node: DomNodeKey, focusable: bool) {
        self.dom_nodes[node].inner.set_focusable(focusable);
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
        if let Some(pos) = focusable_nodes.iter().position(|n| n == key) {
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

    pub fn set_block(&mut self, node: DomNodeKey, block: Block<'static>) {
        self.dom_nodes[node].inner.block = Some(block);
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

    pub(crate) fn add_focusable(&self, key: DomNodeKey) {
        if !self.dom_nodes[key].inner.enabled() {
            return;
        }

        self.focusable_nodes.borrow_mut().push(key);
    }

    pub(crate) fn remove_hovered(&mut self) {
        self.hovered_key = None;
    }

    fn remove_focused(&mut self) {
        if let Some(focused_key) = self.focused_key {
            let mut on_blur = self.dom_nodes[focused_key]
                .inner
                .event_handlers
                .on_blur
                .clone();
            let rect = *self.dom_nodes[focused_key].inner.rect.borrow();
            let node_id = self.dom_nodes[focused_key].inner.id.clone();

            for on_blur in &mut on_blur {
                on_blur.borrow_mut()(
                    BlurEvent { new_target: None },
                    EventData {
                        rect,
                        target: node_id.clone(),
                    },
                );
            }
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

    pub fn set_focused_untracked(&mut self, node_key: Option<DomNodeKey>) {
        let Some(node_key) = node_key else {
            self.focused = None;
            self.focused_key = None;
            return;
        };
        if !self.dom_nodes.contains_key(node_key) {
            self.focused = None;
            self.focused_key = None;
            return;
        }
        self.focused_key = Some(node_key);
        self.focused = self.dom_nodes[node_key].inner.id.clone();
    }

    pub fn set_focused(&mut self, node_key: Option<DomNodeKey>) {
        let prev_focused_id = if let Some(focused_key) = self.focused_key {
            let node_id = self.dom_nodes[focused_key].inner.id.clone();
            let mut on_blur = self.dom_nodes[focused_key]
                .inner
                .event_handlers
                .on_blur
                .clone();

            let rect = *self.dom_nodes[focused_key].inner.rect.borrow();

            let event_data = EventData {
                rect,
                target: node_id.clone(),
            };
            for on_blur in &mut on_blur {
                let focus_id = node_key.and_then(|k| self.dom_nodes[k].inner.id.clone());
                on_blur.borrow_mut()(
                    BlurEvent {
                        new_target: focus_id,
                    },
                    event_data.clone(),
                );
            }
            node_id
        } else {
            None
        };

        self.set_focused_untracked(node_key);
        if prev_focused_id != self.focused {
            (self.on_focus_change)(self.focused.clone());
        }
        let Some(node_key) = self.focused_key else {
            refresh_dom();
            return;
        };

        let mut on_focus = self.dom_nodes[node_key]
            .inner
            .event_handlers
            .on_focus
            .clone();

        let node_id = self.dom_nodes[node_key].inner.id.clone();
        let rect = *self.dom_nodes[node_key].inner.rect.borrow();
        let event_data = EventData {
            rect,
            target: node_id,
        };

        for on_focused in &mut on_focus {
            on_focused.borrow_mut()(
                FocusEvent {
                    previous_target: prev_focused_id.clone(),
                },
                event_data.clone(),
            );
        }
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
                .position(|n| n == &focused)
                .unwrap();

            let last = self.focusable_nodes.borrow().len() - 1;
            if current_focused < last {
                let next = self.focusable_nodes.borrow()[current_focused + 1];
                self.set_focused(Some(next));
                return;
            }
        }
        let next = self.focusable_nodes.borrow().first().cloned();
        if let Some(next) = next {
            self.set_focused(Some(next));
        }
    }

    pub(crate) fn focus_prev(&mut self) {
        if let Some(focused) = self.focused_key {
            let current_focused = self
                .focusable_nodes
                .borrow()
                .iter()
                .position(|n| n == &focused)
                .unwrap();
            if current_focused > 0 {
                let prev = self.focusable_nodes.borrow()[current_focused - 1];
                self.set_focused(Some(prev));
                return;
            }
        }
        let prev = self.focusable_nodes.borrow().last().cloned();
        if let Some(prev) = prev {
            self.set_focused(Some(prev));
        }
    }
}

pub fn focus_next() {
    with_nodes_mut(|n| n.focus_next())
}

pub fn focus_prev() {
    with_nodes_mut(|n| n.focus_prev())
}

pub fn clear_focus() {
    with_nodes_mut(|n| n.set_focused(None))
}
