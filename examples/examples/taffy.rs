use rooibos::dom::flex_node::taffy::{Dimension, FlexDirection, PrintTree};
use slotmap::{DefaultKey, SlotMap};
use taffy::{
    compute_block_layout, compute_cached_layout, compute_flexbox_layout, compute_grid_layout,
    compute_hidden_layout, compute_leaf_layout, compute_root_layout, AvailableSpace, Cache,
    Display, Layout, LayoutInput, LayoutOutput, LayoutPartialTree, NodeId, Size, Style,
    TraversePartialTree, TraverseTree,
};

struct NodeData {
    pub style: Style,
    pub cache: Cache,
    pub layout: Layout,
    pub dirty: bool,
}

impl NodeData {
    pub const fn new(style: Style) -> Self {
        Self {
            style,
            cache: Cache::new(),
            layout: Layout::new(),
            dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.cache.clear();
        self.dirty = true;
    }
}

/// Custom taffy layout tree
///
/// Most of this is taken directly from [`taffy::TaffyTree`] but
/// we have an additional pass where all node layouts are computed
/// relative to the root node
///
/// See the original `TaffyTree` implementation for details
pub struct LayoutTree {
    nodes: SlotMap<DefaultKey, NodeData>,
    children: SlotMap<DefaultKey, Vec<NodeId>>,
    parents: SlotMap<DefaultKey, Option<NodeId>>,
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::new(),
            children: SlotMap::new(),
            parents: SlotMap::new(),
        }
    }

    pub fn new_leaf(&mut self, layout: Style) -> NodeId {
        let id = self.nodes.insert(NodeData::new(layout));
        let _ = self.children.insert(Vec::new());
        let _ = self.parents.insert(None);

        id.into()
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        let parent_key = parent.into();
        let child_key = child.into();
        self.parents[child_key] = Some(parent);
        self.children[parent_key].push(child);
        self.mark_dirty(parent);
    }

    pub fn new_with_children(&mut self, layout: Style, children: &[NodeId]) -> NodeId {
        let id = NodeId::from(self.nodes.insert(NodeData::new(layout)));

        for child in children {
            self.parents[(*child).into()] = Some(id);
        }

        let _ = self
            .children
            .insert(children.to_vec());
        let _ = self.parents.insert(None);

        id
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        fn mark_dirty_recursive(
            nodes: &mut SlotMap<DefaultKey, NodeData>,
            parents: &SlotMap<DefaultKey, Option<NodeId>>,
            node_key: DefaultKey,
        ) {
            nodes[node_key].mark_dirty();

            if let Some(Some(node)) = parents.get(node_key) {
                mark_dirty_recursive(nodes, parents, (*node).into());
            }
        }

        mark_dirty_recursive(&mut self.nodes, &self.parents, node.into());
    }

    // This is where we customize things
    pub fn compute_layout(&mut self, root_node: NodeId, available_space: Size<AvailableSpace>) {
        // Compute tree layout starting from the root node
        compute_root_layout(self, root_node, available_space);

        // Go through each node and adjust its layout relative to its parent
        // so that at the end all nodes are relative to the root
        fn adjust_recursive(
            nodes: &mut SlotMap<DefaultKey, NodeData>,
            parents: &SlotMap<DefaultKey, Option<NodeId>>,
            children: &SlotMap<DefaultKey, Vec<NodeId>>,
            parent_node: NodeId,
        ) {
            if !nodes[parent_node.into()].dirty {
                return;
            }

            nodes[parent_node.into()].dirty = false;

            let parent_layout = nodes[parent_node.into()].layout;
            let children_list = &children[parent_node.into()];

            // This node has children, adjust their layouts and recurse to each one
            for &child in children_list.iter() {
                let child_layout = &mut nodes[child.into()].layout;

                child_layout.location.x += parent_layout.location.x;
                child_layout.location.y += parent_layout.location.y;
            }

            for &child in children_list.iter() {
                adjust_recursive(nodes, parents, children, child);
            }
        }

        adjust_recursive(&mut self.nodes, &self.parents, &self.children, root_node);
    }

    pub fn layout(&self, node: NodeId) -> &Layout {
        &self.nodes[node.into()].layout
    }

    pub fn print_tree(&mut self, root: NodeId) {
        taffy::print_tree(self, root)
    }
}

impl PrintTree for LayoutTree {
    fn get_debug_label(&self, node_id: rooibos::dom::flex_node::taffy::NodeId) -> &'static str {
        let node = &self.nodes[node_id.into()];
        let display = node.style.display;
        let num_children = self.child_count(node_id);

        match (num_children, display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",

            (_, Display::Block) => "BLOCK",

            (_, Display::Flex) => {
                use crate::FlexDirection;
                match node.style.flex_direction {
                    FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                    FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
                }
            }

            (_, Display::Grid) => "GRID",
        }
    }

    fn get_final_layout(
        &self,
        node_id: rooibos::dom::flex_node::taffy::NodeId,
    ) -> &rooibos::dom::flex_node::taffy::Layout {
        &self.nodes[node_id.into()].layout
    }
}

pub struct LayoutTreeChildIter<'a>(std::slice::Iter<'a, NodeId>);

impl<'a> Iterator for LayoutTreeChildIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}

impl TraversePartialTree for LayoutTree {
    type ChildIter<'a> = LayoutTreeChildIter<'a> where Self: 'a;

    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        LayoutTreeChildIter(self.children[parent_node_id.into()].iter())
    }

    fn child_count(&self, parent_node_id: NodeId) -> usize {
        self.children[parent_node_id.into()].len()
    }

    fn get_child_id(&self, parent_node_id: NodeId, id: usize) -> NodeId {
        self.children[parent_node_id.into()][id]
    }
}

// TraverseTree impl for TaffyTree
impl TraverseTree for LayoutTree {}

impl LayoutPartialTree for LayoutTree {
    fn get_style(&self, node: NodeId) -> &Style {
        &self.nodes[node.into()].style
    }

    fn set_unrounded_layout(&mut self, node: NodeId, layout: &Layout) {
        self.nodes[node.into()].layout = *layout;
    }

    fn get_cache_mut(&mut self, node: NodeId) -> &mut Cache {
        &mut self.nodes[node.into()].cache
    }

    fn compute_child_layout(&mut self, node: NodeId, inputs: LayoutInput) -> LayoutOutput {
        compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let display_mode = tree.get_style(node).display;
            let has_children = tree.child_count(node) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node
            // has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, node),
                (Display::Block, true) => compute_block_layout(tree, node, inputs),
                (Display::Flex, true) => compute_flexbox_layout(tree, node, inputs),
                (Display::Grid, true) => compute_grid_layout(tree, node, inputs),

                (_, false) => {
                    let node_key = node.into();
                    let style = &tree.nodes[node_key].style;
                    let measure_function = |_, _| Size::ZERO;
                    compute_leaf_layout(inputs, style, measure_function)
                }
            }
        })
    }
}

fn main() {
    let mut tree: LayoutTree = LayoutTree::new();

    let leaf1 = tree.new_leaf(Style {
        display: Display::Flex,
        size: Size::<Dimension> {
            width: Dimension::Length(50.0),
            height: Dimension::Length(4.0),
        },
        ..Default::default()
    });

    let leaf2 = tree.new_leaf(Style {
        display: Display::Flex,
        flex_grow: 1.0,
        size: Size::<Dimension> {
            width: Dimension::Length(50.0),
            height: Dimension::Length(1.0),
        },
        ..Default::default()
    });
    let row = tree.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_grow: 1.0,
            size: Size::<Dimension> {
                width: Dimension::Length(50.0),
                height: Dimension::Length(1.0),
            },
            ..Default::default()
        },
        &[leaf2],
    );
    let col = tree.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size::<Dimension> {
                width: Dimension::Length(100.0),
                height: Dimension::Length(10.0),
            },

            ..Default::default()
        },
        &[leaf1, row],
    );

    // tree.add_child(col, leaf1).unwrap();
    // tree.add_child(col, row).unwrap();
    // tree.add_child(row, leaf2).unwrap();

    tree.compute_layout(
        col,
        Size::<AvailableSpace> {
            width: AvailableSpace::Definite(120.0),
            height: AvailableSpace::Definite(22.0),
        },
    );
    println!("{:?}", tree.layout(leaf2));
    tree.print_tree(col);
}
