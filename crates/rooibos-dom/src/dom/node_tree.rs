use std::cell::RefCell;
use std::cmp;
use std::collections::BTreeMap;
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::Block;
use slotmap::{new_key_type, SlotMap};
use taffy::{
    AvailableSpace, Dimension, Display, FlexDirection, NodeId, Point, Position, Size, Style,
    TaffyTree,
};

use super::{dom_node, refresh_dom, with_nodes, AsDomNode, DomNode};
use crate::{DomNodeInner, EventHandlers};

new_key_type! { pub(crate) struct DomNodeKey; }

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

pub(crate) struct TreeValue {
    pub(crate) inner: DomNodeInner,
    layout_id: NodeId,
}

#[derive(Clone)]
struct Context {
    offset: Point<f32>,
    width_auto: bool,
    height_auto: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            width_auto: true,
            height_auto: true,
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

pub(crate) struct NodeTree {
    dom_nodes: SlotMap<DomNodeKey, TreeValue>,
    layout_tree: TaffyTree<Context>,
    roots: BTreeMap<RootId, Box<dyn AsDomNode>>,
}

impl Index<DomNodeKey> for NodeTree {
    type Output = DomNodeInner;
    fn index(&self, index: DomNodeKey) -> &Self::Output {
        &self.dom_nodes[index].inner
    }
}

impl NodeTree {
    pub(crate) fn new() -> Self {
        Self {
            layout_tree: TaffyTree::<Context>::new(),
            roots: BTreeMap::default(),
            dom_nodes: Default::default(),
        }
    }

    pub(crate) fn recompute_layout(&mut self, rect: Rect) {
        let root_keys: Vec<_> = self.roots.values().map(|r| r.as_dom_node().key()).collect();
        for root_key in root_keys {
            let node = &self.dom_nodes[root_key];
            self.layout_tree
                .compute_layout(
                    node.layout_id,
                    Size::<AvailableSpace> {
                        width: AvailableSpace::Definite(rect.width.into()),
                        height: AvailableSpace::Definite(rect.height.into()),
                    },
                )
                .unwrap();
            self.recompute_offsets(root_key);
        }
    }

    pub(crate) fn set_root(&mut self, z_index: i32, root: Box<dyn AsDomNode>) {
        let key = root.as_dom_node().key();
        self.roots.insert(RootId::new(z_index), root);
        let node = &self.dom_nodes[key];
        let mut style = self.layout_tree.style(node.layout_id).unwrap().clone();
        if style.size.width == Dimension::Auto {
            style.size.width = Dimension::Percent(1.);
        }
        if style.size.height == Dimension::Auto {
            style.size.height = Dimension::Percent(1.);
        }
        self.layout_tree.set_style(node.layout_id, style).unwrap();
    }

    pub(crate) fn root(&self, z_index: i32) -> &dyn AsDomNode {
        let key = &self.roots.keys().find(|k| k.z_index == z_index).unwrap();
        &self.roots[key]
    }

    pub(crate) fn roots_asc(&self) -> Vec<DomNode> {
        self.roots
            .values()
            .map(|r| r.as_dom_node().clone())
            .collect()
    }

    pub(crate) fn roots_desc(&self) -> Vec<DomNode> {
        let mut roots: Vec<_> = self
            .roots
            .values()
            .map(|r| r.as_dom_node().clone())
            .collect();
        roots.reverse();
        roots
    }

    fn recompute_offsets(&mut self, root_key: DomNodeKey) {
        let node = &self.dom_nodes[root_key];
        self.recompute_offset(node.layout_id);
    }

    fn recompute_offset(&mut self, parent: NodeId) {
        let parent_context = self.layout_tree.get_node_context(parent).unwrap().clone();
        let children = self.layout_tree.children(parent).unwrap();
        for child in &children {
            let child_layout = *self.layout_tree.layout(*child).unwrap();
            let Some(context) = self.layout_tree.get_node_context_mut(*child) else {
                panic!()
            };
            let new_x = child_layout.location.x + parent_context.offset.x;
            let new_y = child_layout.location.y + parent_context.offset.y;
            // if context.0.x != new_x || context.0.y != new_y {
            context.offset.x = new_x;
            context.offset.y = new_y;
            self.recompute_offset(*child);
            // }
        }
    }

    pub(crate) fn update_layout<F>(&mut self, node: DomNodeKey, mut f: F)
    where
        F: FnMut(&mut Style),
    {
        let value = &self.dom_nodes[node];
        let mut style = self.layout_tree.style(value.layout_id).unwrap().clone();
        let before = style.clone();
        f(&mut style);
        let context = self
            .layout_tree
            .get_node_context_mut(value.layout_id)
            .unwrap();
        if style.size.width != before.size.width {
            context.width_auto = false;
        }
        if style.size.height != before.size.height {
            context.height_auto = false;
        }
        self.layout_tree
            .set_style(value.layout_id, style.clone())
            .unwrap();
        let parent = self.layout_tree.parent(value.layout_id);
        if let Some(parent) = parent {
            if before.display != style.display {
                self.update_sizes(parent);
            }
        }

        refresh_dom();
    }

    pub(crate) fn rect(&self, key: DomNodeKey) -> Rect {
        let context = self
            .layout_tree
            .get_node_context(self.dom_nodes[key].layout_id)
            .unwrap();
        computed_to_rect(
            self.layout_tree
                .layout(self.dom_nodes[key].layout_id)
                .unwrap(),
            context,
        )
    }

    pub(crate) fn style(&self, key: DomNodeKey) -> taffy::Style {
        self.layout_tree
            .style(self.dom_nodes[key].layout_id)
            .unwrap()
            .clone()
    }

    fn update_sizes(&mut self, parent: NodeId) {
        let parent_style = self.layout_tree.style(parent).unwrap().clone();

        let children = self.layout_tree.children(parent).unwrap();
        let num_children = children
            .iter()
            .filter(|c| {
                let style = self.layout_tree.style(**c).unwrap();
                style.display != Display::None && style.position != Position::Absolute
            })
            .count() as f32;
        for child in children {
            let mut style = self.layout_tree.style(child).unwrap().clone();
            if style.display != Display::None && style.position != Position::Absolute {
                let context = self.layout_tree.get_node_context(child).unwrap();

                if parent_style.display == Display::Block
                    || (parent_style.display == Display::Flex
                        && parent_style.flex_direction == FlexDirection::Row)
                {
                    if context.width_auto {
                        style.size.width = Dimension::Percent(1. / num_children);
                    }
                    if context.height_auto {
                        style.size.height = Dimension::Percent(1.);
                    }
                } else if parent_style.display == Display::Flex
                    && parent_style.flex_direction == FlexDirection::Column
                {
                    if context.height_auto {
                        style.size.height = Dimension::Percent(1. / num_children);
                    }
                    if context.width_auto {
                        style.size.width = Dimension::Percent(1.);
                    }
                }
                self.layout_tree.set_style(child, style).unwrap();
            }
            self.update_sizes(child);
        }
    }

    pub(crate) fn print_layout_tree(&mut self) {
        let key = self.root(0).as_dom_node().key();
        let layout_id = self.dom_nodes[key].layout_id;
        self.layout_tree.print_tree(layout_id);
    }

    pub(crate) fn keys(&self) -> slotmap::basic::Keys<'_, DomNodeKey, TreeValue> {
        self.dom_nodes.keys()
    }

    pub(crate) fn iter_nodes(&self) -> slotmap::basic::Iter<'_, DomNodeKey, TreeValue> {
        self.dom_nodes.iter()
    }

    pub(crate) fn contains_key(&self, key: DomNodeKey) -> bool {
        self.dom_nodes.contains_key(key)
    }

    pub(crate) fn insert(&mut self, val: DomNodeInner) -> DomNodeKey {
        let layout_node = self
            .layout_tree
            .new_leaf_with_context(Style::default(), Context::default())
            .unwrap();
        self.dom_nodes.insert(TreeValue {
            inner: val,
            layout_id: layout_node,
        })
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
                self.update_sizes(parent_node.layout_id);
            }
        } else {
            self.dom_nodes[parent_key].inner.children.push(child_key);
            self.dom_nodes[child_key].inner.parent = Some(parent_key);
            let parent_node = &self.dom_nodes[parent_key];
            let child_node = &self.dom_nodes[child_key];
            self.layout_tree
                .add_child(parent_node.layout_id, child_node.layout_id)
                .unwrap();
            self.update_sizes(parent_node.layout_id);
        }
    }

    pub(crate) fn remove(&mut self, node: DomNodeKey) -> Option<TreeValue> {
        let layout_id = self.dom_nodes[node].layout_id;
        let parent = self.layout_tree.parent(layout_id);
        self.layout_tree.remove(layout_id).unwrap();
        if let Some(parent) = parent {
            self.update_sizes(parent);
        }

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
            self.update_sizes(parent_node.layout_id);
        }
    }

    pub(crate) fn replace_node(&mut self, old_key: DomNodeKey, new_key: DomNodeKey) {
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
            original_display,
            block,
            z_index: _z_index,
            unmounted: _unmounted,
        } = &self.dom_nodes[old_key].inner;
        // let layout_id = self.dom_nodes[old_key].layout_id;
        let name = name.clone();
        let node_type = node_type.clone();
        let constraint = constraint.clone();
        let focusable = focusable.clone();
        let id = id.clone();
        let class = class.clone();
        let event_handlers = event_handlers.clone();
        let rect = rect.clone();
        let original_display = *original_display;
        let block = block.clone();

        let new = &mut self.dom_nodes[new_key].inner;

        new.name = name;
        new.node_type = node_type;
        new.constraint = constraint;
        new.focusable = focusable;
        new.id = id;
        new.class = class;
        new.event_handlers = event_handlers;
        new.rect = rect;
        new.original_display = original_display;
        new.block = block;
        // self.dom_nodes[new_key].layout_id = layout_id;
    }

    pub(crate) fn replace_inner(&mut self, node: DomNodeKey, inner: DomNodeInner) {
        self.dom_nodes[node].inner = inner;
    }

    pub(crate) fn set_focusable(&mut self, node: DomNodeKey, focusable: Rc<RefCell<bool>>) {
        self.dom_nodes[node].inner.focusable = focusable;
    }

    pub(crate) fn update_event_handlers<F>(&mut self, node: DomNodeKey, update: F)
    where
        F: FnOnce(EventHandlers) -> EventHandlers,
    {
        self.dom_nodes[node].inner.event_handlers =
            update(self.dom_nodes[node].inner.event_handlers.clone())
    }

    pub(crate) fn set_id(&mut self, node: DomNodeKey, id: impl Into<dom_node::NodeId>) {
        self.dom_nodes[node].inner.id = Some(id.into());
    }

    pub(crate) fn set_class(&mut self, node: DomNodeKey, class: impl Into<String>) {
        self.dom_nodes[node].inner.class = Some(class.into());
    }

    pub(crate) fn set_block(&mut self, node: DomNodeKey, block: Block<'static>) {
        self.dom_nodes[node].inner.block = Some(block);
    }

    pub(crate) fn set_z_index(&mut self, key: DomNodeKey, z_index: i32) {
        self.dom_nodes[key].inner.z_index = Some(z_index);
        let unmounted = self.dom_nodes[key].inner.unmounted.clone();
        let node = DomNode::from_existing(key, unmounted);
        self.roots.insert(RootId::new(z_index), Box::new(node));
    }
}

fn computed_to_rect(computed: &taffy::Layout, context: &Context) -> Rect {
    Rect {
        x: context.offset.x as u16,
        y: context.offset.y as u16,
        width: computed.size.width as u16,
        height: computed.size.height as u16,
    }
}
