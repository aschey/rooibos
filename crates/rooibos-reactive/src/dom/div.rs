use next_tuple::NextTuple;
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::AsDomNode;
use tachys::prelude::Renderer;
use tachys::view::{Mountable, Render};
pub use taffy;

use super::layout::{
    AspectRatio, BorderProp, Clear, Height, Margin, MarginBottom, MarginLeft, MarginRight,
    MarginTop, MarginX, MarginY, MaxHeight, MaxWidth, MinHeight, MinWidth, Padding, PaddingBottom,
    PaddingLeft, PaddingRight, PaddingTop, PaddingX, PaddingY, Position, Property, Show, Width,
    ZIndex, aspect_ratio, height, margin, margin_bottom, margin_left, margin_right, margin_top,
    margin_x, margin_y, max_height, max_width, min_height, min_width, padding, padding_bottom,
    padding_left, padding_right, padding_top, padding_x, padding_y, position, show, width,
};
use super::{DomNode, RenderAny, RooibosDom};

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

impl<C, P> Div<C, P> {
    pub fn z_index(mut self, z_index: i32) -> Self {
        self.inner.0 = self.inner.0.z_index(z_index);
        self
    }
}

pub fn div<C, P>(props: P, children: C) -> Div<C, P> {
    Div {
        inner: DomNode(rooibos_dom::DomNode::div()),
        children,
        properties: props,
    }
}

pub trait DivProperty: Property {}

impl DivProperty for () {}
impl DivProperty for BorderProp {}
impl DivProperty for ZIndex {}
impl DivProperty for Clear {}

macro_rules! div_prop {
    ($struct_name:ident, $fn:ident, $inner:ty) => {
        impl DivProperty for $struct_name {}

        impl<C, P> Div<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> Div<C, P::Output<$struct_name>>
            where
                S: Into<Signal<$inner>>,
            {
                Div {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($fn(val)),
                }
            }
        }
    };
}

macro_rules! dimension_div_prop {
    ($struct_name:ident, $fn:ident) => {
        impl DivProperty for $struct_name {}

        impl<C, P> Div<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> Div<C, P::Output<$struct_name>>
            where
                S: $crate::dom::layout::IntoDimensionSignal,
            {
                Div {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($fn(val)),
                }
            }
        }
    };
}

dimension_div_prop!(Width, width);
dimension_div_prop!(Height, height);
dimension_div_prop!(MinWidth, min_width);
dimension_div_prop!(MinHeight, min_height);
dimension_div_prop!(MaxWidth, max_width);
dimension_div_prop!(MaxHeight, max_height);
div_prop!(AspectRatio, aspect_ratio, f32);
div_prop!(Position, position, taffy::style::Position);

dimension_div_prop!(MarginLeft, margin_left);
dimension_div_prop!(MarginRight, margin_right);
dimension_div_prop!(MarginTop, margin_top);
dimension_div_prop!(MarginBottom, margin_bottom);
dimension_div_prop!(MarginX, margin_x);
dimension_div_prop!(MarginY, margin_y);
dimension_div_prop!(Margin, margin);

dimension_div_prop!(PaddingLeft, padding_left);
dimension_div_prop!(PaddingRight, padding_right);
dimension_div_prop!(PaddingTop, padding_top);
dimension_div_prop!(PaddingBottom, padding_bottom);
dimension_div_prop!(PaddingX, padding_x);
dimension_div_prop!(PaddingY, padding_y);
dimension_div_prop!(Padding, padding);

div_prop!(Show, show, bool);

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
    fn as_dom_node(&self) -> &rooibos_dom::DomNode {
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
    C: RenderAny,
    P: Property + DivProperty + 'static,
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
