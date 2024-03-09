use std::fmt;
use std::rc::Rc;

use super::component::ComponentRepr;
use super::dom_node::DomNode;
use super::dom_widget::DomWidget;
use super::dyn_child::DynChildRepr;
use super::unit::UnitRepr;
use crate::{next_node_id, EachRepr};

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
    Each(EachRepr),
    Unit(UnitRepr),
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
            View::Each(repr) => {
                repr.set_name(name);
            }
            View::DomNode(node) => {
                node.set_name(name);
            }
            View::DomWidget(widget) => {
                widget.widget_type = name.into();
            }
            View::Unit(_) => {}
        }
    }
}

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl IntoView for &View {
    fn into_view(self) -> View {
        self.clone()
    }
}

impl<const N: usize, IV: IntoView> IntoView for [IV; N] {
    fn into_view(self) -> View {
        ViewFragment::new(self.into_iter().map(|v| v.into_view()).collect()).into_view()
    }
}

pub trait CollectView {
    fn collect_view(self) -> View;
}

impl<I: IntoIterator<Item = T>, T: IntoView> CollectView for I {
    fn collect_view(self) -> View {
        self.into_iter()
            .map(|v| v.into_view())
            .collect::<ViewFragment>()
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
            .collect::<ViewFragment>()
            .into_view()
    }
}

impl IntoView for View {
    fn into_view(self) -> View {
        self
    }
}

pub struct ViewFragment {
    id: u32,
    nodes: Vec<View>,
}

impl ViewFragment {
    pub fn new(nodes: Vec<View>) -> Self {
        Self {
            id: next_node_id(),
            nodes,
        }
    }
}

impl FromIterator<View> for ViewFragment {
    fn from_iter<T: IntoIterator<Item = View>>(iter: T) -> Self {
        ViewFragment::new(iter.into_iter().collect())
    }
}

impl IntoView for ViewFragment {
    fn into_view(self) -> View {
        let repr = ComponentRepr::new_with_id("fragment", self.id, self.nodes);
        repr.into_view()
    }
}

impl Mountable for View {
    fn get_mountable_node(&self) -> DomNode {
        match self {
            Self::DomNode(dom_node) => dom_node.clone(),
            Self::DynChild(dyn_child) => dyn_child.get_mountable_node(),
            Self::Each(each) => each.get_mountable_node(),
            Self::Component(component) => component.get_mountable_node(),
            Self::DomWidget(widget) => widget.get_mountable_node(),
            Self::Unit(unit) => unit.get_mountable_node(),
        }
    }
}

#[derive(Clone)]
pub struct ViewFn(Rc<dyn Fn() -> View>);

impl Default for ViewFn {
    fn default() -> Self {
        Self(Rc::new(|| ().into_view()))
    }
}

impl<F, IV> From<F> for ViewFn
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    fn from(value: F) -> Self {
        Self(Rc::new(move || value().into_view()))
    }
}

impl ViewFn {
    pub fn run(&self) -> View {
        (self.0)()
    }
}

macro_rules! impl_into_view_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty),*> IntoView for ($($ty,)*)
        where $($ty: IntoView),*
        {
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
