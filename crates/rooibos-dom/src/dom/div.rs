use next_tuple::NextTuple;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::Renderer;
use tachys::view::{Mountable, Render};
pub use taffy;

use super::layout::{
    aspect_ratio, border, height, margin, max_height, max_width, min_height, min_width, padding, width, AspectRatio, Border, Height, Margin, MaxHeight, MaxWidth, MinHeight, MinWidth, Padding, Width,
};
use super::{AsDomNode, DomNode, Property, RenderAny, RooibosDom};

#[derive(Debug)]
pub struct Div<C, P> {
    inner: DomNode,
    children: C,
    properties: P,
}

impl<C, P> Div<C, P>
where
    C: NextTuple,
{
    pub fn child<T>(self, child: T) -> Div<C::Output<T>, P> {
        Div {
            inner: self.inner,
            children: self.children.next_tuple(child),
            properties: self.properties,
        }
    }
}

pub fn div<C, P>(props: P, children: C) -> Div<C, P> {
    Div {
        inner: DomNode::div(),
        children,
        properties: props,
    }
}

trait DivProperty {}

impl DivProperty for () {}

macro_rules! div_prop {
    ($struct_name:ident, $fn:ident, $inner:ty) => {
        impl DivProperty for $struct_name {}

        impl<C, P> Div<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> Div<C, P::Output<$struct_name>>
            where
                S: Into<MaybeSignal<$inner>>,
            {
                Div {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($fn(val).0),
                }
            }
        }
    };
}

div_prop!(Width, width, taffy::Dimension);
div_prop!(Height, height, taffy::Dimension);
div_prop!(MinWidth, min_width, taffy::Dimension);
div_prop!(MinHeight, min_height, taffy::Dimension);
div_prop!(MaxWidth, max_width, taffy::Dimension);
div_prop!(MaxHeight, max_height, taffy::Dimension);
div_prop!(AspectRatio, aspect_ratio, f32);
div_prop!(Margin, margin, taffy::Rect<taffy::LengthPercentageAuto>);
div_prop!(Padding, padding, taffy::Rect<taffy::LengthPercentage>);
div_prop!(Border, border, taffy::Rect<taffy::LengthPercentage>);

pub struct DivState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    node: <DomNode as Render<RooibosDom>>::State,
    prop_state: <P as Property>::State,
    children: <C as Render<RooibosDom>>::State,
}

impl<C, P> AsDomNode for DivState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    fn as_dom_node(&self) -> &DomNode {
        self.node.as_dom_node()
    }
}

impl<C, P> Mountable<RooibosDom> for DivState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    fn unmount(&mut self) {
        self.node.unmount();
    }

    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        self.node.mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        self.node.insert_before_this(child)
    }
}

impl<C, P> Render<RooibosDom> for Div<C, P>
where
    C: RenderAny + 'static,
    P: Property + DivProperty,
{
    type State = DivState<C, P>;

    fn build(self) -> Self::State {
        let inner_state = self.inner.build();
        let prop_state = self.properties.build(&inner_state.0);
        let mut children_state = self.children.build();
        children_state.mount(&inner_state.0, None);

        DivState {
            node: inner_state,
            children: children_state,
            prop_state,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        if self.inner == state.node.0 {
            self.inner.rebuild(&mut state.node);
            self.properties
                .rebuild(&state.node.0, &mut state.prop_state);
            self.children.rebuild(&mut state.children);
        } else {
            state.children.unmount();
            let mut children_state = self.children.build();
            children_state.mount(&state.node.0, None);
            state.children = children_state;
        }
    }
}

#[macro_export]
macro_rules! div {
    () => (
        $crate::div::div((), ())
    );
    (props($($properties:expr),+ $(,)?)) => (
        $crate::div::div(($($properties),+), ())
    );
    (props($($properties:expr),+), $($children:expr),+ $(,)?) => (
        $crate::div::div(($($properties),+), ($($children),+))
    );
    (props($($properties:expr),+), $children:expr) => (
        $crate::div::div(($($properties),+), ($children,))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::div::div((), ($($children),+))
    );
}

macro_rules! impl_div_property_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> DivProperty for ($($ty,)*)
            where $($ty: DivProperty,)*
        { }
    }
}

impl_div_property_for_tuples!(A);
impl_div_property_for_tuples!(A, B);
impl_div_property_for_tuples!(A, B, C);
impl_div_property_for_tuples!(A, B, C, D);
impl_div_property_for_tuples!(A, B, C, D, E);
impl_div_property_for_tuples!(A, B, C, D, E, F);
impl_div_property_for_tuples!(A, B, C, D, E, F, G);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_div_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_div_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_div_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_div_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_div_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_div_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_div_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
