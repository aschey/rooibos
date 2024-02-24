use std::cell::{OnceCell, RefCell};
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use leptos_reactive::{create_render_effect, untrack_with_diagnostics};
use ratatui::layout::Flex;
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use slotmap::{new_key_type, SlotMap};

thread_local! {
    pub static DOM: RefCell<DomNode> = RefCell::new(DomNode::root());
    static DOM_NODES: RefCell<SlotMap<DomNodeKey, DomNodeInner>> =
        RefCell::new(SlotMap::<DomNodeKey, DomNodeInner>::default());
}

static NODE_ID: AtomicU32 = AtomicU32::new(1);

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

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
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
    Root,
    Overlay,
    Widget(DomWidget),
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

impl<const N: usize> IntoView for [View; N] {
    fn into_view(self) -> View {
        Fragment::new(self.into_iter().collect()).into_view()
    }
}

impl<V> IntoView for Vec<V>
where
    V: IntoView,
{
    fn into_view(self) -> View {
        self.into_iter()
            .map(|v| v.into_view())
            .collect::<Fragment>()
            .into_view()
    }
}

macro_rules! impl_into_view_for_tuples {
    ($($ty:ident),* $(,)?) => {
      impl<$($ty),*> IntoView for ($($ty,)*)
      where
        $($ty: IntoView),*
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
    pub nodes: Vec<View>,
    pub(crate) view_marker: Option<String>,
}

impl Fragment {
    pub fn new(nodes: Vec<View>) -> Self {
        Self {
            id: NODE_ID.fetch_add(1, Ordering::Relaxed),
            nodes,
            view_marker: None,
        }
    }
}

impl FromIterator<View> for Fragment {
    fn from_iter<T: IntoIterator<Item = View>>(iter: T) -> Self {
        Fragment::new(iter.into_iter().collect())
    }
}

// impl From<View> for Fragment {
//     fn from(view: View) -> Self {
//         Fragment::new(vec![view])
//     }
// }

// impl From<Fragment> for View {
//     fn from(value: Fragment) -> Self {
//         let mut repr = ComponentRepr::new_with_id("".to_string(), value.id, value.nodes);

//         repr.view_marker = value.view_marker;

//         repr.into_view()
//     }
// }

impl IntoView for Fragment {
    fn into_view(self) -> View {
        let mut repr = ComponentRepr::new_with_id("".to_string(), self.id, self.nodes);
        repr.view_marker = self.view_marker;
        repr.into_view()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ComponentRepr {
    pub(crate) document_fragment: DomNode,
    mounted: Rc<OnceCell<DomNode>>,
    pub(crate) name: String,
    opening: Comment,
    pub children: Vec<View>,
    closing: Comment,
    pub(crate) id: u32,
    pub(crate) view_marker: Option<String>,
}

impl ComponentRepr {
    pub fn new_with_id(name: String, id: u32, children: Vec<View>) -> Self {
        let document_fragment = DocumentFragment::transparent();
        let markers = (
            Comment::new(format!("<{name}>"), id, false),
            Comment::new(format!("</{name}>"), id, true),
        );

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            mounted: Default::default(),
            opening: markers.0,
            closing: markers.1,
            name,
            children,
            id,
            view_marker: None,
        }
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
            // mount_child(MountKind::Append(&self.opening.node), &self.closing.node);
            for child in &self.children {
                mount_child(MountKind::Before(&self.closing.node), child);
            }
            let node = self.document_fragment.clone();
            // mount_child(MountKind::Before(&self.closing.node), &node.clone());
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

// #[derive(Clone, PartialEq, Eq)]
// pub struct DomNode {
//     id: Option<String>,
//     node_type: NodeType,
//     constraint: Constraint,
//     children: Vec<DomNode>,
// }

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

// #[derive(Clone)]
// pub struct View {
//     id: u32,
//     f: Rc<RefCell<dyn FnMut(&mut Frame, Rect)>>,
// }

pub struct DynChild<CF, V>
where
    CF: Fn() -> V + 'static,
    V: IntoView,
{
    id: u32,
    child_fn: CF,
}

impl<CF, N> DynChild<CF, N>
where
    CF: Fn() -> N + 'static,
    N: IntoView,
{
    pub fn new(child_fn: CF) -> Self {
        Self {
            child_fn,
            id: NODE_ID.fetch_add(1, Ordering::Relaxed),
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
            let document_fragment = component.document_fragment.clone();
            let opening = component.opening.node.clone();
            let closing = component.closing.node.clone();
            let child = component.child.clone();

            create_render_effect(move |prev_run: Option<Option<DomNodeKey>>| {
                let new_child = child_fn().into_view();
                let mut child_borrow = (*child).borrow_mut();

                // Is this at least the second time we are loading a child?
                if let Some(prev_t) = prev_run {
                    let child = child_borrow.take().unwrap();

                    if child != new_child {
                        let new = mount_child(MountKind::Before(&closing), &new_child);

                        **child_borrow = Some(new_child);
                        Some(new)
                    } else {
                        prev_t
                    }
                } else {
                    let new = mount_child(MountKind::Before(&closing), &new_child);
                    Some(new)
                }
            });
            component
        }

        let Self { id, child_fn } = self;

        let component = DynChildRepr::new_with_id(id);
        let component = create_dyn_view(component, Box::new(move || child_fn().into_view()));

        View::DynChild(component)
    }
    // fn into_view(self) -> View {
    //     let child_fn = self.child_fn;
    //     let prev = leptos_reactive::SpecialNonReactiveZone::enter();
    //     let view = child_fn().into_view();
    //     leptos_reactive::SpecialNonReactiveZone::exit(prev);

    //     create_render_effect(move |prev_id: Option<u32>| {
    //         let new_view = child_fn().into_view();
    //         let new_id = new_view.id;
    //         if let Some(prev_id) = prev_id {
    //             // if prev_id != new_id {
    //             DOM.with(|d| {
    //                 let res = d.borrow().replace(prev_id, new_view);
    //                 // dbg!(res.is_none());
    //             })
    //             // }
    //         }

    //         new_id
    //     });
    // }
}

#[derive(Clone, PartialEq, Eq)]
struct Comment {
    node: DomNode,
    content: String,
}

impl Comment {
    fn new(content: impl Into<String>, id: u32, closing: bool) -> Self {
        let node = DomNode::from_fragment(DocumentFragment::transparent());
        // DOM_NODES.with(|d| d.borrow_mut().insert(node.));
        // mount_child(MountKind::Append(parent), &node);
        // DOM.with(|d| d.borrow().append_child(&node));
        Self {
            content: content.into(),
            node,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DynChildRepr {
    document_fragment: DomNode,
    mounted: Rc<OnceCell<DomNode>>,
    #[cfg(debug_assertions)]
    opening: Comment,
    pub(crate) child: Rc<RefCell<Box<Option<View>>>>,
    closing: Comment,
    pub(crate) id: u32,
}

impl DynChildRepr {
    fn new_with_id(id: u32) -> Self {
        let document_fragment = DocumentFragment::transparent();
        let markers = (
            Comment::new("<DynChild>", id, false),
            Comment::new("</DynChild>", id, true),
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

// impl Eq for View {}

// impl PartialEq for View {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl<F, V> IntoView for F
// where
//     F: Fn() -> V + 'static,
//     V: IntoView,
// {
//     fn into_view(self) -> View {
//         DynChild::new(self).into_view()
//     }
// }

// impl View {
//     pub fn new(f: impl FnMut(&mut Frame, Rect) + 'static) -> Self {
//         Self {
//             id: 0,
//             f: Rc::new(RefCell::new(f)),
//         }
//     }

//     pub fn render(&self, frame: &mut Frame, rect: Rect) {
//         (self.f.borrow_mut())(frame, rect)
//     }
// }

// impl<F: 'static> IntoView for F
// where
//     F: FnMut(&mut Frame, Rect),
// {
//     fn into_view(self) -> View {
//         View::new(self)
//     }
// }

#[derive(Clone)]
pub struct DomWidget {
    f: Rc<RefCell<dyn FnMut(&mut Frame, Rect)>>,
}

impl DomWidget {
    pub fn new<F: FnMut(&mut Frame, Rect) + 'static>(f: F) -> Self {
        Self {
            f: Rc::new(RefCell::new(f)),
        }
    }

    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        (*self.f).borrow_mut()(frame, rect)
    }
}

impl IntoView for DomWidget {
    fn into_view(self) -> View {
        View::DomWidget(self)
    }
}

impl Mountable for DomWidget {
    fn get_mountable_node(&self) -> DomNode {
        DomNode::from_fragment(DocumentFragment::widget(self.clone()))
    }
}

impl PartialEq for DomWidget {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl Eq for DomWidget {}

impl<F, N> IntoView for F
where
    F: Fn() -> N + 'static,
    N: IntoView,
{
    fn into_view(self) -> View {
        DynChild::new(self).into_view()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DocumentFragment {
    node_type: NodeType,
    constraint: Constraint,
    flex: Flex,
    children: Vec<DomNodeKey>,
}

impl DocumentFragment {
    pub fn widget(widget: DomWidget) -> Self {
        Self {
            node_type: NodeType::Widget(widget),
            children: vec![],
            constraint: Constraint::default(),
            flex: Flex::default(),
        }
    }

    pub fn row() -> Self {
        Self {
            node_type: NodeType::Layout {
                direction: Direction::Horizontal,
                flex: Flex::default(),
                margin: 0,
                spacing: 0,
            },
            children: vec![],
            constraint: Constraint::default(),
            flex: Flex::default(),
        }
    }

    pub fn col() -> Self {
        Self {
            node_type: NodeType::Layout {
                direction: Direction::Vertical,
                flex: Flex::default(),
                margin: 0,
                spacing: 0,
            },
            children: vec![],
            constraint: Constraint::default(),
            flex: Flex::default(),
        }
    }

    pub fn overlay() -> Self {
        Self {
            node_type: NodeType::Overlay,
            children: vec![],
            constraint: Constraint::default(),
            flex: Flex::default(),
        }
    }

    pub fn transparent() -> Self {
        Self {
            node_type: NodeType::Transparent,
            children: vec![],
            constraint: Constraint::default(),
            flex: Flex::default(),
        }
    }

    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    pub fn margin(mut self, new_margin: u16) -> Self {
        if let NodeType::Layout { margin, .. } = &mut self.node_type {
            *margin = new_margin;
        }
        self
    }

    // pub fn into_node(self) -> DomNode {
    //     DomNode::from_fragment(self)
    // }
}

pub struct Element {
    inner: DomNode,
}

impl Element {
    pub fn child(self, child: impl IntoView) -> Self {
        let child = child.into_view();
        mount_child(MountKind::Append(&self.inner), &child);
        // DOM_NODES.with(|d| d.borrow_mut()[child_key].constraint = constraint);
        self
    }

    // pub fn constraint(self, constraint: Constraint) -> Self {
    //     DOM_NODES.with(|d| d.borrow_mut()[self.inner.key].constraint = constraint);
    //     self
    // }
}

impl IntoView for Element {
    fn into_view(self) -> View {
        View::DomNode(self.inner)
    }
}

pub fn row(constraint: Constraint) -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::row().constraint(constraint)),
    }
}

pub fn col(constraint: Constraint) -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::col().constraint(constraint)),
    }
}

#[derive(Clone, PartialEq, Eq)]
struct DomNodeInner {
    node_type: NodeType,
    constraint: Constraint,
    children: Vec<DomNodeKey>,
    parent: Option<DomNodeKey>,
    before_pending: Vec<DomNodeKey>,
}

impl DomNodeInner {
    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        DOM_NODES.with(|d| {
            let d = d.borrow();
            let children: Vec<_> = self
                .children
                .iter()
                .map(|c| &d[*c])
                // .flat_map(|c| {
                //     let child = &d[*c];
                //     let mut before: Vec<_> = child.before_pending.iter().map(|c|
                // &d[*c]).collect();     before.push(child);
                //     before
                // })
                // .filter(|c| c.node_type != NodeType::Transparent)
                .collect();
            // let mut grandchildren: Vec<_> = children
            //     .iter()
            //     .filter_map(|c| {
            //         if c.node_type == NodeType::Transparent {
            //             Some(c.children.iter().map(|c| &d[*c]).collect::<Vec<_>>())
            //         } else {
            //             None
            //         }
            //     })
            //     .flatten()
            //     .collect();
            // children.append(&mut grandchildren);
            let constraints = children.iter().map(|c| c.constraint);
            // dbg!(&constraints);
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
                        .for_each(|(child, chunk)| {
                            child.render(frame, *chunk);
                        });
                }
                // NodeType::Transparent => {}
                NodeType::Overlay | NodeType::Transparent | NodeType::Root => {
                    children.iter().for_each(|child| {
                        child.render(frame, rect);
                    });
                }
                NodeType::Widget(widget) => {
                    widget.render(frame, rect);
                }
            }
        });
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DomNode {
    // inner: Rc<RefCell<DomNodeInner>>,
    key: DomNodeKey,
    // parent: Rc<RefCell<Option<DomNodeKey>>>,
}

impl DomNode {
    pub fn root() -> Self {
        let inner = DomNodeInner {
            node_type: NodeType::Root,
            constraint: Constraint::Min(0),
            children: vec![],
            parent: None,
            before_pending: vec![],
        };
        let key = DOM_NODES.with(|n| n.borrow_mut().insert(inner));
        Self {
            // inner: Rc::new(RefCell::new(inner)),
            key,
            // parent: Rc::new(RefCell::new(None)),
        }
    }

    pub fn from_fragment(fragment: DocumentFragment) -> Self {
        let inner = DomNodeInner {
            node_type: fragment.node_type,
            constraint: fragment.constraint,
            children: fragment.children,
            parent: None,
            before_pending: vec![],
        };
        let key = DOM_NODES.with(|n| n.borrow_mut().insert(inner));
        Self {
            // inner:Rc::new(RefCell::new(inner)),
            key,
            // parent: Rc::new(RefCell::new(None)),
        }
    }

    pub fn append_child(&self, node: &DomNode) {
        // *(*node.parent).borrow_mut() = Some(self.key);
        DOM_NODES.with(|d| {
            let mut d = d.borrow_mut();
            // let dn = &d[node.key];
            // dbg!(&dn.before_pending);
            // let ds = &d[self.key];
            // dbg!(&ds.before_pending);
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
                d[node.key].parent = Some(self.key);
            }
        });
        // DOM_NODES.with(|d| d.borrow_mut()[self.key].children.push(node.key));
        // (*self.inner).borrow_mut().children.push(node.key);
    }

    pub fn before(&self, node: &DomNode) {
        // if let Some(parent_id) = self.parent.borrow().as_ref() {
        DOM_NODES.with(|d| {
            let mut d = d.borrow_mut();
            // let dn = &d[node.key];
            // dbg!(&dn.before_pending);
            // let ds = &d[self.key];
            // dbg!(&ds.before_pending);
            if let Some(parent_id) = d[self.key].parent {
                let parent = d.get_mut(parent_id).unwrap();
                let self_index = parent.children.iter().position(|c| c == &self.key).unwrap();
                parent.children.insert(self_index, node.key);
                d[node.key].parent = Some(parent_id);
                // let pending = d[self.key].before_pending.drain(..);
                // node.parent = self.parent.clone();
            } else {
                d[self.key].before_pending.push(node.key);
            }
        });
        // }
    }

    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        DOM_NODES.with(|d| d.borrow()[self.key].render(frame, rect));
        // self.inner.borrow().render(frame, rect);
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

// impl<F> IntoView for DomWidget<F>
// where
//     F: FnMut(&mut Frame, Rect) + 'static,
// {
//     fn into_view(self) -> View {
//         View::new(self.inner)
//     }
// }

// impl IntoView for View {
//     fn into_view(self) -> View {
//         self
//     }
// }

// impl IntoView for DomNode {
//     fn into_view(self) -> View {
//         View::new(move |frame, rect| self.render(frame, rect))
//     }
// }

// pub trait IntoDomNode {
//     fn attach(self, children: &mut Vec<DomNode>);
// }

// impl IntoDomNode for DomNode {
//     fn attach(self, children: &mut Vec<DomNode>) {
//         children.push(self);
//     }
// }

// impl IntoDomNode for View {
//     fn attach(self, children: &mut Vec<DomNode>) {
//         children.push(DomNode::component(self));
//     }
// }

// impl IntoDomNode for Vec<DomNode> {
//     fn attach(self, children: &mut Vec<DomNode>) {
//         for node in self.into_iter() {
//             children.push(node);
//         }
//     }
// }

// impl<F: 'static> IntoDomNode for DomWidget<F>
// where
//     F: FnMut(&mut Frame, Rect),
// {
//     fn attach(self, children: &mut Vec<DomNode>) {
//         self.into_view().attach(children);
//     }
// }

// impl DomNode {
//     fn root() -> Self {
//         Self {
//             id: None,
//             node_type: NodeType::Root,
//             children: vec![],
//             constraint: Constraint::Min(0),
//         }
//     }

//     pub fn component(v: View) -> Self {
//         Self {
//             id: None,
//             node_type: NodeType::Component(RefCell::new(v.into_view())),
//             children: vec![],
//             constraint: Constraint::Min(0),
//         }
//     }

//     pub fn row() -> Self {
//         Self {
//             id: None,
//             node_type: NodeType::Layout {
//                 direction: Direction::Horizontal,
//                 margin: 0,
//             },
//             children: vec![],
//             constraint: Constraint::Min(0),
//         }
//     }

//     pub fn col() -> Self {
//         Self {
//             id: None,
//             node_type: NodeType::Layout {
//                 direction: Direction::Vertical,
//                 margin: 0,
//             },
//             children: vec![],
//             constraint: Constraint::Min(0),
//         }
//     }

//     pub fn overlay() -> Self {
//         Self {
//             id: None,
//             node_type: NodeType::Overlay,
//             children: vec![],
//             constraint: Constraint::Min(0),
//         }
//     }

//     pub fn constraint(mut self, constraint: Constraint) -> Self {
//         self.constraint = constraint;
//         self
//     }

//     pub fn render(&self, frame: &mut Frame, rect: Rect) {
//         match &self.node_type {
//             NodeType::Layout { direction, margin } => {
//                 let layout = Layout::default().direction(*direction).margin(*margin);
//                 let constraints = self.children.iter().map(|c| c.constraint);
//                 let chunks = layout
//                     .constraints(constraints.collect::<Vec<_>>())
//                     .split(rect);
//                 self.children
//                     .iter()
//                     .zip(chunks.iter())
//                     .for_each(|(child, chunk)| {
//                         child.render(frame, *chunk);
//                     });
//             }
//             NodeType::Overlay | NodeType::Root => {
//                 self.children.iter().for_each(|child| {
//                     child.render(frame, rect);
//                 });
//             }
//             NodeType::Component(component) => {
//                 component.borrow().render(frame, rect);
//             }
//         }
//     }

//     pub fn margin(mut self, new_margin: u16) -> Self {
//         if let NodeType::Layout { margin, .. } = &mut self.node_type {
//             *margin = new_margin;
//         }
//         self
//     }

//     pub fn child(mut self, node: impl IntoDomNode) -> Self {
//         node.attach(&mut self.children);
//         self
//     }

//     fn matches_id(&self, id: u32) -> bool {
//         if let NodeType::Component(component) = &self.node_type {
//             return component.borrow().id == id;
//         }
//         false
//     }

//     fn replace_view(&self, view: View) {
//         if let NodeType::Component(component) = &self.node_type {
//             println!("HI");
//             *component.borrow_mut() = view;
//         }
//     }

//     pub fn replace(&self, id: u32, mut new: View) -> Option<View> {
//         if self.matches_id(id) {
//             self.replace_view(new);
//             return None;
//         }
//         for child in self.children.iter() {
//             match child.replace(id, new) {
//                 Some(returned) => {
//                     new = returned;
//                 }
//                 None => {
//                     return None;
//                 }
//             }
//         }
//         Some(new)
//     }
// }

pub fn mount(v: impl IntoView) {
    let node = v.into_view().get_mountable_node();
    DOM.with(|d| d.borrow_mut().append_child(&node));
}

pub fn render_dom(frame: &mut Frame) {
    DOM.with(|d| d.borrow().render(frame, frame.size()));
}
