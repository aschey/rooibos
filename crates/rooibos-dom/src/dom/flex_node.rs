use next_tuple::NextTuple;
use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::Renderer;
use tachys::view::{Mountable, Render};
pub use taffy;

use super::{with_nodes_mut, AsDomNode, DomNode, Property, RenderAny, RooibosDom};
use crate::WidgetProperty;

#[derive(Debug)]
pub struct FlexNode<C, P> {
    inner: DomNode,
    children: C,
    properties: P,
}

impl<C, P> FlexNode<C, P>
where
    C: NextTuple,
{
    pub fn child<T>(self, child: T) -> FlexNode<C::Output<T>, P> {
        FlexNode {
            inner: self.inner,
            children: self.children.next_tuple(child),
            properties: self.properties,
        }
    }
}

pub fn row<C, P>(props: P, children: C) -> FlexNode<C, P> {
    FlexNode {
        inner: DomNode::flex_row(),
        children,
        properties: props,
    }
}

pub fn col<C, P>(props: P, children: C) -> FlexNode<C, P> {
    FlexNode {
        inner: DomNode::flex_col(),
        children,
        properties: props,
    }
}

trait UpdateLayout {
    fn update_layout(&self, style: &mut taffy::Style);
}

impl<T> Property for T
where
    T: UpdateLayout + 'static,
{
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| nodes.update_layout(key, |s| self.update_layout(s)));
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

trait FlexProperty {}

impl FlexProperty for () {}

macro_rules! layout_prop {
    ($struct_name:ident, $fn:ident, $inner:ty, $($prop:ident).*) => {
        pub struct $struct_name(MaybeSignal<$inner>);

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, style: &mut taffy::Style) {
                style.$($prop).* = self.0.get();
            }
        }

        pub fn $fn(val: impl Into<MaybeSignal<$inner>>) -> ($struct_name,) {
            ($struct_name(val.into()),)
        }

        impl<C, P> FlexNode<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> FlexNode<C, P::Output<$struct_name>>
            where
                S: Into<MaybeSignal<$inner>>,
            {
                FlexNode {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($struct_name(val.into())),
                }
            }
        }
    };
}

macro_rules! layout_prop_opt {
    ($struct_name:ident, $fn:ident, $inner:ty, $($prop:ident).*) => {
        pub struct $struct_name(MaybeSignal<$inner>);

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, style: &mut taffy::Style) {
                style.$($prop).* = Some(self.0.get());
            }
        }

        pub fn $fn(val: impl Into<MaybeSignal<$inner>>) -> ($struct_name,) {
            ($struct_name(val.into()),)
        }

        impl<C, P> FlexNode<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> FlexNode<C, P::Output<$struct_name>>
            where
                S: Into<MaybeSignal<$inner>>,
            {
                FlexNode {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($struct_name(val.into())),
                }
            }
        }
    };
}

layout_prop!(Width, width, taffy::Dimension, size.width);
impl WidgetProperty for Width {}
impl FlexProperty for Width {}
layout_prop!(Height, height, taffy::Dimension, size.height);
impl WidgetProperty for Height {}
impl FlexProperty for Height {}

layout_prop!(Wrap, wrap, taffy::FlexWrap, flex_wrap);
impl FlexProperty for Wrap {}
layout_prop_opt!(AlignItems, align_items, taffy::AlignItems, align_items);
impl FlexProperty for AlignItems {}
layout_prop_opt!(
    AlignContent,
    align_content,
    taffy::AlignContent,
    align_content
);
impl FlexProperty for AlignContent {}
layout_prop_opt!(
    JustifyContent,
    justify_content,
    taffy::JustifyContent,
    justify_content
);
impl FlexProperty for JustifyContent {}
layout_prop!(Gap, gap, taffy::Size<taffy::LengthPercentage>, gap);
impl FlexProperty for Gap {}

layout_prop!(Grow, grow, f32, flex_grow);
impl WidgetProperty for Grow {}
impl FlexProperty for Grow {}
layout_prop!(Shrink, shrink, f32, flex_shrink);
impl WidgetProperty for Shrink {}
impl FlexProperty for Shrink {}
layout_prop_opt!(AlignSelf, align_self, taffy::AlignSelf, align_self);
impl WidgetProperty for AlignSelf {}
impl FlexProperty for AlignSelf {}
layout_prop!(Basis, basis, taffy::Dimension, flex_basis);
impl WidgetProperty for Basis {}
impl FlexProperty for Basis {}

pub fn chars(val: f32) -> taffy::Dimension {
    taffy::Dimension::Length(val)
}

pub fn pct(val: f32) -> taffy::Dimension {
    taffy::Dimension::Percent(val / 100.0)
}

pub struct FlexNodeState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    node: <DomNode as Render<RooibosDom>>::State,
    prop_state: <P as Property>::State,
    children: <C as Render<RooibosDom>>::State,
}

impl<C, P> AsDomNode for FlexNodeState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    fn as_dom_node(&self) -> &DomNode {
        self.node.as_dom_node()
    }
}

impl<C, P> Mountable<RooibosDom> for FlexNodeState<C, P>
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

impl<C, P> Render<RooibosDom> for FlexNode<C, P>
where
    C: RenderAny + 'static,
    P: Property + FlexProperty,
{
    type State = FlexNodeState<C, P>;

    fn build(self) -> Self::State {
        let inner_state = self.inner.build();
        let prop_state = self.properties.build(&inner_state.0);
        let mut children_state = self.children.build();
        children_state.mount(&inner_state.0, None);

        FlexNodeState {
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
macro_rules! flex_row {
    () => (
        $crate::flex_node::row((), ())
    );
    (props($($properties:expr),+ $(,)?)) => (
        $crate::flex_node::row(($($properties),+), ())
    );
    (props($($properties:expr),+), $($children:expr),+ $(,)?) => (
        $crate::flex_node::row(($($properties),+), ($($children),+))
    );
    (props($($properties:expr),+), $children:expr) => (
        $crate::flex_node::row(($($properties),+), ($children,))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::flex_node::row((), ($($children),+))
    );
}

#[macro_export]
macro_rules! flex_col {
    () => (
        $crate::flex_node::col((), ())
    );
    (props($($properties:expr),+ $(,)?)) => (
        $crate::flex_node::col(($($properties),+), ())
    );
    (props($($properties:expr),+), $($children:expr),+ $(,)?) => (
        $crate::flex_node::col(($($properties),+), ($($children),+))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::flex_node::col((), ($($children),+))
    );
}

macro_rules! impl_flex_property_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> FlexProperty for ($($ty,)*)
            where $($ty: FlexProperty,)*
        { }
    }
}

impl_flex_property_for_tuples!(A);
impl_flex_property_for_tuples!(A, B);
impl_flex_property_for_tuples!(A, B, C);
impl_flex_property_for_tuples!(A, B, C, D);
impl_flex_property_for_tuples!(A, B, C, D, E);
impl_flex_property_for_tuples!(A, B, C, D, E, F);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_flex_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_flex_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_flex_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_flex_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_flex_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_flex_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_flex_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
