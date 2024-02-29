use std::borrow::Borrow;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use ratatui::layout::Flex;
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use rooibos_reactive::{
    create_render_effect, create_signal, untrack_with_diagnostics, ReadSignal, SignalSet,
    SignalUpdate, WriteSignal,
};
use slotmap::{new_key_type, SlotMap};

// Reference for focus impl https://github.com/reactjs/rfcs/pull/109/files

thread_local! {
    static DOM_ROOT: RefCell<Option<DomNode>> = RefCell::new(None);
    static DOM_NODES: RefCell<SlotMap<DomNodeKey, DomNodeInner>> =
        RefCell::new(SlotMap::<DomNodeKey, DomNodeInner>::default());
    static DOM_STATE: RefCell<DomState> = RefCell::new(Default::default());
}

static NODE_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, PartialEq, Eq)]
pub enum NodeIdInner {
    Auto(u32),
    Manual(String),
}

#[derive(Clone, PartialEq, Eq)]
pub struct NodeId(NodeIdInner);

impl NodeId {
    pub fn new_auto() -> Self {
        Self(NodeIdInner::Auto(NODE_ID.fetch_add(1, Ordering::Relaxed)))
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

new_key_type! {pub struct FocusHandlerKey; }

struct DomState {
    focused: ReadSignal<Option<NodeId>>,
    set_focused: WriteSignal<Option<NodeId>>,
    focused_key: Option<DomNodeKey>,
}

impl Default for DomState {
    fn default() -> Self {
        let (focused, set_focused) = create_signal(None);
        Self {
            focused,
            set_focused,
            focused_key: None,
        }
    }
}

impl DomState {
    fn set_focused(&mut self, node_key: Option<DomNodeKey>, node: &DomNodeInner) {
        self.focused_key = node_key;
        self.set_focused.set(node.id.to_owned());
    }
}

pub trait IntoView {
    fn into_view(self) -> View;
}

pub trait Mountable {
    fn get_mountable_node(&self) -> DomNode;
}

#[derive(Clone, PartialEq, Eq)]
pub enum View {
    DynChild(DynChildRepr),
    Component(ComponentRepr),
    DomNode(DomNode),
    DomWidget(DomWidget),
}

impl View {
    fn set_name(&mut self, name: impl Into<String>) {
        match self {
            View::DynChild(repr) => {
                repr.set_name(name);
            }
            View::Component(repr) => {
                repr.set_name(name);
            }
            View::DomNode(node) => {
                node.set_name(name);
            }
            View::DomWidget(widget) => {
                widget.widget_type = name.into();
            }
        }
    }
}

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

struct NodeTypeStructure {
    name: &'static str,
    attrs: Option<String>,
    children: Option<String>,
}
#[derive(Clone, PartialEq, Eq)]
enum NodeType {
    Layout {
        direction: Direction,
        flex: Flex,
        margin: u16,
        spacing: u16,
    },
    Transparent,
    Overlay,
    Widget(DomWidget),
}

impl NodeType {
    fn structure(&self) -> NodeTypeStructure {
        match self {
            NodeType::Layout {
                direction,
                flex,
                margin,
                spacing,
            } => NodeTypeStructure {
                name: "Layout",
                attrs: Some(format!(
                    "direction={direction}, flex={flex}, margin={margin}, spacing={spacing}"
                )),
                children: None,
            },

            NodeType::Transparent => NodeTypeStructure {
                name: "Transparent",
                attrs: None,
                children: None,
            },
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
            NodeType::Layout {
                direction,
                flex,
                margin,
                spacing,
            } => write!(
                f,
                "Layout(direction={direction}, flex={flex}, margin={margin}, spacing={spacing})"
            ),

            NodeType::Transparent => write!(f, "Transparent"),
            // NodeType::Root => write!(f, "Root"),
            NodeType::Overlay => write!(f, "Overlay"),
            NodeType::Widget(widget) => write!(f, "Widget({widget:?})"),
        }
    }
}

new_key_type! {struct DomNodeKey; }

impl IntoView for View {
    fn into_view(self) -> View {
        self
    }
}

impl Mountable for View {
    fn get_mountable_node(&self) -> DomNode {
        match self {
            Self::DomNode(dom_node) => dom_node.clone(),
            Self::DynChild(dyn_child) => dyn_child.get_mountable_node(),
            Self::Component(component) => component.get_mountable_node(),
            Self::DomWidget(widget) => widget.get_mountable_node(),
        }
    }
}

impl IntoView for &View {
    fn into_view(self) -> View {
        self.clone()
    }
}

impl<const N: usize, IV: IntoView> IntoView for [IV; N] {
    fn into_view(self) -> View {
        Fragment::new(self.into_iter().map(|v| v.into_view()).collect()).into_view()
    }
}

pub trait CollectView {
    fn collect_view(self) -> View;
}

impl<I: IntoIterator<Item = T>, T: IntoView> CollectView for I {
    fn collect_view(self) -> View {
        self.into_iter()
            .map(|v| v.into_view())
            .collect::<Fragment>()
            .into_view()
    }
}

impl<IV> IntoView for Vec<IV>
where
    IV: IntoView,
{
    fn into_view(self) -> View {
        self.into_iter()
            .map(|v| v.into_view())
            .collect::<Fragment>()
            .into_view()
    }
}

impl<IV> IntoView for (&'static str, IV)
where
    IV: IntoView,
{
    fn into_view(self) -> View {
        let mut view = self.1.into_view();
        view.set_name(self.0);
        view
    }
}

macro_rules! impl_into_view_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty),*> IntoView for ($($ty,)*)
        where $($ty: IntoView),*
        {
            #[inline]
            fn into_view(self) -> View {
                paste::paste! {
                    let ($([<$ty:lower>],)*) = self;
                    [
                    $([<$ty:lower>].into_view()),*
                    ].into_view()
                }
            }
        }
    };
}

impl_into_view_for_tuples!(A);
impl_into_view_for_tuples!(A, B);
impl_into_view_for_tuples!(A, B, C);
impl_into_view_for_tuples!(A, B, C, D);
impl_into_view_for_tuples!(A, B, C, D, E);
impl_into_view_for_tuples!(A, B, C, D, E, F);
impl_into_view_for_tuples!(A, B, C, D, E, F, G);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_into_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_into_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_into_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_into_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_into_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_into_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_into_view_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

pub struct Fragment {
    id: u32,
    nodes: Vec<View>,
    // pub(crate) view_marker: Option<String>,
}

impl Fragment {
    pub fn new(nodes: Vec<View>) -> Self {
        Self {
            id: NODE_ID.fetch_add(1, Ordering::Relaxed),
            nodes,
            // view_marker: None,
        }
    }
}

impl FromIterator<View> for Fragment {
    fn from_iter<T: IntoIterator<Item = View>>(iter: T) -> Self {
        Fragment::new(iter.into_iter().collect())
    }
}

impl IntoView for Fragment {
    fn into_view(self) -> View {
        let repr = ComponentRepr::new_with_id("fragment", self.id, self.nodes);
        // repr.view_marker = self.view_marker;
        repr.into_view()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ComponentRepr {
    document_fragment: DomNode,
    mounted: Rc<OnceCell<DomNode>>,
    opening: Comment,
    children: Vec<View>,
    closing: Comment,
    id: u32,
}

impl ComponentRepr {
    pub fn new_with_id(name: impl Into<String>, id: u32, children: Vec<View>) -> Self {
        let name = name.into();
        let document_fragment = DocumentFragment::transparent(name.clone());
        let markers = (
            Comment::new(name.clone(), id, false),
            Comment::new(name, id, true),
        );

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            mounted: Default::default(),
            opening: markers.0,
            closing: markers.1,
            children,
            id,
        }
    }

    fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.opening.set_name(name.clone());
        self.closing.set_name(name);
    }
}

impl Mountable for ComponentRepr {
    fn get_mountable_node(&self) -> DomNode {
        if let Some(mounted) = self.mounted.get() {
            mounted.clone()
        } else {
            mount_child(
                MountKind::Append(&self.document_fragment),
                &self.opening.node,
            );
            mount_child(
                MountKind::Append(&self.document_fragment),
                &self.closing.node,
            );

            for child in &self.children {
                mount_child(MountKind::Before(&self.closing.node), child);
            }
            let node = self.document_fragment.clone();
            self.mounted.set(node.clone()).unwrap();
            node
        }
    }
}

impl IntoView for ComponentRepr {
    fn into_view(self) -> View {
        View::Component(self)
    }
}

pub enum MountKind<'a> {
    Before(&'a DomNode),
    Append(&'a DomNode),
}

fn mount_child<M: Mountable + std::fmt::Debug>(kind: MountKind, child: &M) -> DomNodeKey {
    let child = child.get_mountable_node();

    match kind {
        MountKind::Append(el) => {
            el.append_child(&child);
        }
        MountKind::Before(closing) => {
            closing.before(&child);
        }
    }
    child.key
}

fn cleanup_removed_nodes(
    node: &DomNodeKey,
    nodes: &mut RefMut<'_, SlotMap<DomNodeKey, DomNodeInner>>,
) {
    let children = nodes[*node].children.clone();
    for child in children {
        cleanup_removed_nodes(&child, nodes);
    }
    nodes.remove(*node);
}

fn unmount_child(child: DomNodeKey) {
    DOM_NODES.with(|d| {
        let mut d = d.borrow_mut();
        let child_node = &d[child];
        if let Some(parent) = child_node.parent {
            let child_pos = d[parent].children.iter().position(|c| c == &child).unwrap();
            d[parent].children.remove(child_pos);
        }

        cleanup_removed_nodes(&child, &mut d);
    });
}

pub struct Component<F, V>
where
    F: FnOnce() -> V,
    V: IntoView,
{
    id: u32,
    name: String,
    children_fn: F,
}

impl<F, V> Component<F, V>
where
    F: FnOnce() -> V,
    V: IntoView,
{
    /// Creates a new component.
    pub fn new(name: impl Into<String>, f: F) -> Self {
        Self {
            id: NODE_ID.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            children_fn: f,
        }
    }
}

impl<F, V> IntoView for Component<F, V>
where
    F: FnOnce() -> V,
    V: IntoView,
{
    fn into_view(self) -> View {
        let Self {
            id,
            name,
            children_fn,
        } = self;

        // disposed automatically when the parent scope is disposed
        let child = untrack_with_diagnostics(|| children_fn().into_view());
        let repr = ComponentRepr::new_with_id(name, id, vec![child]);

        repr.into_view()
    }
}

pub struct DynChild<CF, V>
where
    CF: Fn() -> V + 'static,
    V: IntoView,
{
    id: u32,
    child_fn: CF,
    name: String,
}

impl<CF, N> DynChild<CF, N>
where
    CF: Fn() -> N + 'static,
    N: IntoView,
{
    pub fn new(name: impl Into<String>, child_fn: CF) -> Self {
        Self {
            child_fn,
            id: NODE_ID.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
        }
    }
}

impl<CF, N> IntoView for DynChild<CF, N>
where
    CF: Fn() -> N + 'static,
    N: IntoView,
{
    fn into_view(self) -> View {
        fn create_dyn_view(
            component: DynChildRepr,
            child_fn: Box<dyn Fn() -> View>,
        ) -> DynChildRepr {
            let closing = component.closing.node.clone();
            let child = component.child.clone();

            create_render_effect(move |prev_key: Option<DomNodeKey>| {
                let new_child = child_fn().into_view();
                let mut child_borrow = (*child).borrow_mut();

                // Is this at least the second time we are loading a child?
                if let Some(prev_key) = prev_key {
                    let prev_child = child_borrow.take().unwrap();

                    if prev_child != new_child {
                        unmount_child(prev_key);

                        let new_key = mount_child(MountKind::Before(&closing), &new_child);

                        **child_borrow = Some(new_child);
                        new_key
                    } else {
                        prev_key
                    }
                } else {
                    let new = mount_child(MountKind::Before(&closing), &new_child);
                    **child_borrow = Some(new_child);
                    new
                }
            });
            component
        }

        let Self { id, child_fn, name } = self;

        let component = DynChildRepr::new_with_id(id, name);
        let component = create_dyn_view(component, Box::new(move || child_fn().into_view()));

        View::DynChild(component)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Comment {
    node: DomNode,
}

impl Comment {
    fn new(name: impl Into<String>, id: u32, closing: bool) -> Self {
        let node = DomNode::from_fragment(DocumentFragment::transparent(name));

        Self { node }
    }

    fn set_name(&mut self, name: impl Into<String>) {
        self.node.set_name(name);
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DynChildRepr {
    document_fragment: DomNode,
    mounted: Rc<OnceCell<DomNode>>,
    // #[cfg(debug_assertions)]
    opening: Comment,
    pub(crate) child: Rc<RefCell<Box<Option<View>>>>,
    closing: Comment,
    pub(crate) id: u32,
}

impl DynChildRepr {
    fn new_with_id(id: u32, name: impl Into<String>) -> Self {
        let document_fragment = DocumentFragment::transparent(name);
        let markers = (
            Comment::new("DynChild", id, false),
            Comment::new("DynChild", id, true),
        );

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            opening: markers.0,
            closing: markers.1,
            child: Default::default(),
            id,
            mounted: Default::default(),
        }
    }

    fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.opening.set_name(name.clone());
        self.closing.set_name(name);
    }
}

impl Mountable for DynChildRepr {
    fn get_mountable_node(&self) -> DomNode {
        if let Some(mounted) = self.mounted.get() {
            mounted.clone()
        } else {
            mount_child(
                MountKind::Append(&self.document_fragment),
                &self.opening.node,
            );
            mount_child(
                MountKind::Append(&self.document_fragment),
                &self.closing.node,
            );
            self.mounted.set(self.document_fragment.clone()).unwrap();
            self.document_fragment.clone()
        }
    }
}

#[derive(Clone)]
pub struct DomWidget {
    f: Rc<RefCell<dyn FnMut(&mut Frame, Rect)>>,
    id: u32,
    widget_type: String,
    constraint: Constraint,
    dom_id: Option<NodeId>,
}

impl Debug for DomWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}/>", self.widget_type)
    }
}

impl DomWidget {
    pub fn new<F: FnMut(&mut Frame, Rect) + 'static>(
        id: u32,
        widget_type: impl Into<String>,
        f: F,
    ) -> Self {
        Self {
            widget_type: widget_type.into(),
            id,
            f: Rc::new(RefCell::new(f)),
            constraint: Constraint::default(),
            dom_id: None,
        }
    }

    fn render(&self, frame: &mut Frame, rect: Rect) {
        (*self.f).borrow_mut()(frame, rect)
    }

    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.dom_id = Some(id.into());
        self
    }
}

impl IntoView for DomWidget {
    fn into_view(self) -> View {
        View::DomWidget(self)
    }
}

impl Mountable for DomWidget {
    fn get_mountable_node(&self) -> DomNode {
        DomNode::from_fragment(
            DocumentFragment::widget(self.clone())
                .constraint(self.constraint)
                .id(self.dom_id.clone()),
        )
    }
}

impl PartialEq for DomWidget {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomWidget {}

impl<F, N> IntoView for F
where
    F: Fn() -> N + 'static,
    N: IntoView,
{
    fn into_view(self) -> View {
        DynChild::new("Fn", self).into_view()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DocumentFragment {
    node_type: NodeType,
    constraint: Constraint,
    id: Option<NodeId>,
    flex: Flex,
    name: String,
}

impl DocumentFragment {
    fn widget(widget: DomWidget) -> Self {
        Self {
            name: widget.widget_type.clone(),
            constraint: widget.constraint,
            node_type: NodeType::Widget(widget),
            flex: Flex::default(),
            id: None,
        }
    }

    fn row() -> Self {
        Self {
            node_type: NodeType::Layout {
                direction: Direction::Horizontal,
                flex: Flex::default(),
                margin: 0,
                spacing: 0,
            },
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "row".to_string(),
            id: None,
        }
    }

    fn col() -> Self {
        Self {
            node_type: NodeType::Layout {
                direction: Direction::Vertical,
                flex: Flex::default(),
                margin: 0,
                spacing: 0,
            },
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "col".to_string(),
            id: None,
        }
    }

    fn overlay() -> Self {
        Self {
            node_type: NodeType::Overlay,
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "overlay".to_string(),
            id: None,
        }
    }

    fn transparent(name: impl Into<String>) -> Self {
        Self {
            node_type: NodeType::Transparent,
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: name.into(),
            id: None,
        }
    }

    fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    fn id(mut self, id: Option<NodeId>) -> Self {
        self.id = id;
        self
    }
}

pub struct Element {
    inner: DomNode,
}

impl Element {
    pub fn child(self, child: impl IntoView) -> Self {
        let child = child.into_view();
        mount_child(MountKind::Append(&self.inner), &child);
        self
    }

    pub fn constraint(self, constraint: Constraint) -> Self {
        self.inner.set_constraint(constraint);
        self
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn margin(self, margin: u16) -> Self {
        self.inner.set_margin(margin);
        self
    }
}

impl IntoView for Element {
    fn into_view(self) -> View {
        View::DomNode(self.inner)
    }
}

pub fn row() -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::row()),
    }
}

pub fn col() -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::col()),
    }
}

pub fn overlay() -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::overlay()),
    }
}

pub fn print_dom<W: io::Write>(writer: &mut W, include_transparent: bool) -> io::Result<()> {
    DOM_ROOT.with(|dom| {
        DOM_NODES.with(|nodes| {
            let dom = dom.borrow();
            let nodes = nodes.borrow();
            let root = &nodes[dom.as_ref().unwrap().key];
            if !include_transparent && root.node_type == NodeType::Transparent {
                for (key, _) in &root.resolve_children(&nodes) {
                    print_dom_inner(writer, &nodes, *key, "", include_transparent)?;
                }
            } else {
                print_dom_inner(
                    writer,
                    &nodes,
                    dom.as_ref().unwrap().key,
                    "",
                    include_transparent,
                )?;
            }

            Ok(())
        })

        // for node in d.keys() {
        //     print_dom_inner(writer, &d, &d[node], "")?;
        // }
    })
}

fn print_dom_inner<W: io::Write>(
    writer: &mut W,
    dom_ref: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    key: DomNodeKey,
    indent: &str,
    include_transparent: bool,
) -> io::Result<()> {
    // if matches!(node.node_type, NodeType::Transparent) {
    //     for child in &node.children {
    //         print_dom_inner(writer, dom_ref, &dom_ref[*child], indent)?;
    //     }
    // } else {
    let node = &dom_ref[key];
    let NodeTypeStructure {
        name,
        attrs,
        children,
    } = node.node_type.structure();
    let node_name = node.name.clone();
    write!(
        writer,
        "{indent}<{node_name} type={name} key={key:?} parent={:?}",
        node.parent
    )?;
    if let Some(attrs) = attrs {
        write!(writer, " {attrs}")?;
    }
    write!(writer, " constraint={}", node.constraint)?;

    writeln!(writer, ">")?;
    if let Some(children) = children {
        writeln!(writer, "{indent}  {children}")?;
    }
    let child_indent = format!("{indent}  ");
    if include_transparent {
        for key in &node.children {
            print_dom_inner(writer, dom_ref, *key, &child_indent, include_transparent)?;
        }
    } else {
        for (key, _) in &node.resolve_children(dom_ref) {
            print_dom_inner(writer, dom_ref, *key, &child_indent, include_transparent)?;
        }
    }

    writeln!(writer, "{indent}</{node_name}>")?;
    // }

    Ok(())
}

#[derive(Clone, PartialEq, Eq)]
struct DomNodeInner {
    node_type: NodeType,
    name: String,
    constraint: Constraint,
    children: Vec<DomNodeKey>,
    parent: Option<DomNodeKey>,
    before_pending: Vec<DomNodeKey>,
    id: Option<NodeId>,
    focusable: bool,
}

impl DomNodeInner {
    fn resolve_children(
        &self,
        dom_nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    ) -> Vec<(DomNodeKey, DomNodeInner)> {
        let children: Vec<_> = self
            .children
            .iter()
            .flat_map(|c| {
                let child = &dom_nodes[*c];
                if child.node_type == NodeType::Transparent {
                    return child.resolve_children(dom_nodes);
                }
                vec![(*c, child.to_owned())]
            })
            .collect();
        children
    }

    fn render(
        &self,
        frame: &mut Frame,
        rect: Rect,
        dom_nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    ) {
        let children: Vec<_> = self.resolve_children(dom_nodes);

        let constraints = children.iter().map(|(_, c)| c.constraint);

        match &self.node_type {
            NodeType::Layout {
                direction,
                margin,
                flex,
                spacing,
            } => {
                let layout = Layout::default()
                    .direction(*direction)
                    .flex(*flex)
                    .margin(*margin)
                    .spacing(*spacing)
                    .constraints(constraints);

                let chunks = layout.split(rect);
                children
                    .iter()
                    .zip(chunks.iter())
                    .for_each(|((_, child), chunk)| {
                        child.render(frame, *chunk, dom_nodes);
                    });
            }

            NodeType::Overlay | NodeType::Transparent => {
                children.iter().for_each(|(_, child)| {
                    child.render(frame, rect, dom_nodes);
                });
            }
            NodeType::Widget(widget) => {
                widget.render(frame, rect);
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DomNode {
    key: DomNodeKey,
}

impl DomNode {
    fn from_fragment(fragment: DocumentFragment) -> Self {
        let inner = DomNodeInner {
            name: fragment.name.clone(),
            node_type: fragment.node_type,
            constraint: fragment.constraint,
            children: vec![],
            parent: None,
            before_pending: vec![],
            focusable: fragment.id.is_some(),
            id: fragment.id,
        };
        let key = DOM_NODES.with(|n| n.borrow_mut().insert(inner));
        Self { key }
    }

    fn set_name(&self, name: impl Into<String>) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].name = name.into());
    }

    fn set_constraint(&self, constraint: Constraint) {
        DOM_NODES.with(|n| n.borrow_mut()[self.key].constraint = constraint);
    }

    fn set_id(&self, id: impl Into<NodeId>) {
        DOM_NODES.with(|n| {
            let mut n = n.borrow_mut();
            n[self.key].id = Some(id.into());
            n[self.key].focusable = true;
        });
    }

    fn set_margin(&self, new_margin: u16) {
        DOM_NODES.with(|n| {
            if let NodeType::Layout { margin, .. } = &mut n.borrow_mut()[self.key].node_type {
                *margin = new_margin;
            }
        });
    }

    fn append_child(&self, node: &DomNode) {
        // *(*node.parent).borrow_mut() = Some(self.key);
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

    fn before(&self, node: &DomNode) {
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

    fn render(&self, frame: &mut Frame, rect: Rect) {
        DOM_NODES.with(|d| {
            let d = d.borrow();
            d[self.key].render(frame, rect, &d);
        });
    }
}

impl IntoView for DomNode {
    fn into_view(self) -> View {
        View::DomNode(self)
    }
}

impl Mountable for DomNode {
    fn get_mountable_node(&self) -> DomNode {
        self.clone()
    }
}

pub fn mount<F, IV>(f: F)
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView,
{
    let node = f().into_view().get_mountable_node();
    DOM_ROOT.with(|d| *d.borrow_mut() = Some(node));
}

pub fn render_dom(frame: &mut Frame) {
    DOM_ROOT.with(|d| d.borrow().as_ref().unwrap().render(frame, frame.size()));
}

pub fn focus(id: impl Into<NodeId>) {
    let id = id.into();
    let node = DOM_NODES.with(|d| {
        d.borrow().iter().find_map(|(k, v)| {
            if v.id.as_ref() == Some(&id) {
                Some(k)
            } else {
                None
            }
        })
    });
    if let Some(node) = node {
        DOM_STATE.with(|state| {
            DOM_NODES.with(|nodes| {
                state
                    .borrow_mut()
                    .set_focused(Some(node), &nodes.borrow()[node]);
            });
        });
    }
}

fn dfs(
    nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    key: DomNodeKey,
    search_fn: &impl Fn(&DomNodeInner, DomNodeKey) -> bool,
) -> Option<DomNodeKey> {
    let mut visited = HashSet::new();
    dfs_inner(nodes, key, search_fn, &mut visited)
}

fn dfs_inner(
    nodes: &Ref<'_, SlotMap<DomNodeKey, DomNodeInner>>,
    key: DomNodeKey,
    search_fn: &impl Fn(&DomNodeInner, DomNodeKey) -> bool,
    visited: &mut HashSet<DomNodeKey>,
) -> Option<DomNodeKey> {
    visited.insert(key);
    let node = &nodes[key];
    if search_fn(node, key) {
        return Some(key);
    }
    for child in &node.children {
        if !visited.contains(child) {
            if let Some(key) = dfs_inner(nodes, *child, search_fn, visited) {
                return Some(key);
            }
        }
    }
    if let Some(parent) = node.parent {
        let child_index = nodes[parent]
            .children
            .iter()
            .position(|n| n == &key)
            .unwrap();
        for i in (child_index + 1)..nodes[parent].children.len() {
            let child = nodes[parent].children[i];
            if !visited.contains(&child) {
                if let Some(key) = dfs_inner(nodes, child, search_fn, visited) {
                    return Some(key);
                }
            }
        }
        return dfs_inner(nodes, parent, search_fn, visited);
    }
    None
}

pub fn focused_node() -> ReadSignal<Option<NodeId>> {
    DOM_STATE.with(|d| d.borrow().focused)
}

pub fn focus_id(id: impl Into<NodeId>) {
    let id = id.into();
    DOM_NODES.with(|nodes| {
        let nodes = nodes.borrow();
        let found_node = nodes.iter().find_map(|(key, node)| {
            if let Some(current_id) = &node.id {
                if &id == current_id {
                    return Some(key);
                }
            }
            None
        });
        if let Some(found_node) = found_node {
            DOM_STATE.with(|state| {
                state
                    .borrow_mut()
                    .set_focused(Some(found_node), &nodes[found_node]);
            });
        }
    });
}

pub fn focus_next() {
    let focused = DOM_STATE
        .with(|d| d.borrow().focused_key)
        .unwrap_or_else(|| DOM_ROOT.with(|d| d.borrow().as_ref().cloned().unwrap().key));

    DOM_NODES.with(|nodes| {
        let nodes = nodes.borrow();
        let new_focusable = dfs(&nodes, focused, &|node, key| {
            key != focused && node.focusable
        });
        if let Some(focusable) = new_focusable {
            DOM_STATE.with(|state| {
                state
                    .borrow_mut()
                    .set_focused(Some(focusable), &nodes[focusable]);
            })
        } else {
            // Nothing found, start from the top
            let root = DOM_ROOT.with(|r| r.borrow().as_ref().cloned()).unwrap();
            let new_focusable = dfs(&nodes, root.key, &|node, _| node.focusable);
            if let Some(focusable) = new_focusable {
                DOM_STATE.with(|d| {
                    d.borrow_mut()
                        .set_focused(Some(focusable), &nodes[focusable]);
                });
            }
        }
    });
}
