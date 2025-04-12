use next_tuple::NextTuple;
use ratatui::layout::Rect;
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::events::{
    BlurEvent, ClickHandler, EventData, FocusEvent, IntoClickHandler, IntoKeyHandler, KeyHandler,
};
use rooibos_dom::{AsDomNode, Borders, NodeId};
use tachys::prelude::Renderer;
use tachys::view::{Mountable, Render};
pub use taffy;

use super::layout::{
    AlignContent, AlignItems, AlignSelf, AspectRatio, BorderProp, Class, Clear, ColumnGap,
    FlexBasis, FlexGrow, FlexShrink, FlexWrap, Gap, Height, Id, IntoAlignContentSignal,
    IntoAlignItemsSignal, IntoAlignSelfSignal, IntoJustifyContentSignal, IntoJustifyItemsSignal,
    JustifyContent, JustifyItems, Margin, MarginBottom, MarginLeft, MarginRight, MarginTop,
    MarginX, MarginY, MaxHeight, MaxWidth, MinHeight, MinWidth, Overflow, OverflowX, OverflowY,
    Padding, PaddingBottom, PaddingLeft, PaddingRight, PaddingTop, PaddingX, PaddingY, Position,
    Property, RowGap, Show, Width, ZIndex, align_content, align_items, align_self, aspect_ratio,
    borders, class, clear, column_gap, flex_basis, flex_grow, flex_shrink, flex_wrap, focusable,
    gap, height, id, justify_content, justify_items, margin, margin_bottom, margin_left,
    margin_right, margin_top, margin_x, margin_y, max_height, max_width, min_height, min_width,
    overflow, overflow_x, overflow_y, padding, padding_bottom, padding_left, padding_right,
    padding_top, padding_x, padding_y, position, row_gap, show, width, z_index,
};
#[cfg(feature = "effects")]
use super::layout::{Effect, effect};
use super::{DomNode, RenderAny, RooibosDom};
use crate::dom::layout::Focusable;

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

impl<C, P> FlexNode<C, P> {
    pub fn on_focus<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.inner.0 = self
            .inner
            .0
            .on_focus(move |event, data| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler(event, data);
            })
            .focusable(true);
        self
    }

    pub fn on_blur<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.inner.0 = self
            .inner
            .0
            .on_blur(move |event, data| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler(event, data);
            })
            .focusable(true);
        self
    }

    pub fn on_key_down<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        let mut handler = handler.into_key_handler();
        self.inner.0 = self.inner.0.on_key_down(move |props| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler.handle(props)
        });

        self
    }

    pub fn on_key_up<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        let mut handler = handler.into_key_handler();
        self.inner.0 = self.inner.0.on_key_up(move |props| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler.handle(props);
        });
        self
    }

    pub fn on_size_change<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.inner.0 = self.inner.0.on_size_change(move |size| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(size);
        });
        self
    }

    pub fn on_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        let mut handler = handler.into_click_handler();
        self.inner.0 = self.inner.0.on_click(move |props| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler.handle(props)
        });
        self
    }

    pub fn on_right_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        let mut handler = handler.into_click_handler();
        self.inner.0 = self.inner.0.on_right_click(move |props| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler.handle(props)
        });
        self
    }

    pub fn on_middle_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        let mut handler = handler.into_click_handler();
        self.inner.0 = self.inner.0.on_middle_click(move |props| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler.handle(props);
        });
        self
    }
}

pub fn row<C, P>(props: P, children: C) -> FlexNode<C, P> {
    FlexNode {
        inner: DomNode(rooibos_dom::DomNode::flex_row()),
        children,
        properties: props,
    }
}

pub fn col<C, P>(props: P, children: C) -> FlexNode<C, P> {
    FlexNode {
        inner: DomNode(rooibos_dom::DomNode::flex_col()),
        children,
        properties: props,
    }
}

pub trait FlexProperty: Property {}

impl FlexProperty for () {}

impl FlexProperty for Id {}

impl<C, P> FlexNode<C, P>
where
    P: NextTuple,
{
    pub fn id<S>(self, val: S) -> FlexNode<C, P::Output<Id>>
    where
        S: Into<NodeId>,
    {
        FlexNode {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(id(val)),
        }
    }
}

macro_rules! flex_prop {
    ($struct_name:ident, $fn:ident, $inner:ty) => {
        impl FlexProperty for $struct_name {}

        impl<C, P> FlexNode<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> FlexNode<C, P::Output<$struct_name>>
            where
                S: Into<Signal<$inner>>,
            {
                FlexNode {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($fn(val)),
                }
            }
        }
    };
}

macro_rules! custom_flex_prop {
    ($struct_name:ident, $fn:ident, $bound:path) => {
        impl FlexProperty for $struct_name {}

        impl<C, P> FlexNode<C, P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> FlexNode<C, P::Output<$struct_name>>
            where
                S: $bound,
            {
                FlexNode {
                    inner: self.inner,
                    children: self.children,
                    properties: self.properties.next_tuple($fn(val)),
                }
            }
        }
    };
}

macro_rules! dimension_flex_prop {
    ($struct_name:ident, $fn:ident) => {
        custom_flex_prop!($struct_name, $fn, $crate::dom::layout::IntoDimensionSignal);
    };
}

dimension_flex_prop!(Width, width);
dimension_flex_prop!(Height, height);
dimension_flex_prop!(MinWidth, min_width);
dimension_flex_prop!(MinHeight, min_height);
dimension_flex_prop!(MaxWidth, max_width);
dimension_flex_prop!(MaxHeight, max_height);
flex_prop!(AspectRatio, aspect_ratio, f32);
flex_prop!(Position, position, taffy::style::Position);

dimension_flex_prop!(MarginLeft, margin_left);
dimension_flex_prop!(MarginRight, margin_right);
dimension_flex_prop!(MarginTop, margin_top);
dimension_flex_prop!(MarginBottom, margin_bottom);
dimension_flex_prop!(MarginX, margin_x);
dimension_flex_prop!(MarginY, margin_y);
dimension_flex_prop!(Margin, margin);

dimension_flex_prop!(PaddingLeft, padding_left);
dimension_flex_prop!(PaddingRight, padding_right);
dimension_flex_prop!(PaddingTop, padding_top);
dimension_flex_prop!(PaddingBottom, padding_bottom);
dimension_flex_prop!(PaddingX, padding_x);
dimension_flex_prop!(PaddingY, padding_y);
dimension_flex_prop!(Padding, padding);

dimension_flex_prop!(RowGap, row_gap);
dimension_flex_prop!(ColumnGap, column_gap);
dimension_flex_prop!(Gap, gap);

flex_prop!(BorderProp, borders, Borders);
flex_prop!(Focusable, focusable, bool);
flex_prop!(Show, show, bool);
flex_prop!(Clear, clear, bool);
flex_prop!(Class, class, Vec<String>);
flex_prop!(ZIndex, z_index, i32);
#[cfg(feature = "effects")]
flex_prop!(Effect, effect, super::layout::SyncEffect);

flex_prop!(FlexWrap, flex_wrap, taffy::FlexWrap);
custom_flex_prop!(AlignItems, align_items, IntoAlignItemsSignal);
custom_flex_prop!(AlignContent, align_content, IntoAlignContentSignal);
custom_flex_prop!(JustifyItems, justify_items, IntoJustifyItemsSignal);
custom_flex_prop!(JustifyContent, justify_content, IntoJustifyContentSignal);
flex_prop!(FlexGrow, flex_grow, f32);
flex_prop!(FlexShrink, flex_shrink, f32);
custom_flex_prop!(AlignSelf, align_self, IntoAlignSelfSignal);
dimension_flex_prop!(FlexBasis, flex_basis);

flex_prop!(OverflowX, overflow_x, taffy::Overflow);
flex_prop!(OverflowY, overflow_y, taffy::Overflow);
flex_prop!(Overflow, overflow, taffy::Overflow);

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
    fn as_dom_node(&self) -> &rooibos_dom::DomNode {
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
    C: RenderAny,
    P: FlexProperty + 'static,
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
macro_rules! row {
    () => (
        $crate::dom::flex_node::row((), ())
    );
    (style($($properties:expr),+ $(,)?)) => (
        $crate::dom::flex_node::row(($($properties),+), ())
    );
    (style($($properties:expr),+ $(,)?), $($children:expr),+ $(,)?) => (
        $crate::dom::flex_node::row(($($properties),+), ($($children),+))
    );
    (style($($properties:expr),+ $(,)?), $children:expr) => (
        $crate::dom::flex_node::row(($($properties),+), ($children,))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::dom::flex_node::row((), ($($children),+))
    );
}

#[macro_export]
macro_rules! col {
    () => (
        $crate::dom::flex_node::col((), ())
    );
    (style($($properties:expr),+ $(,)?)) => (
        $crate::dom::flex_node::col(($($properties),+), ())
    );
    (style($($properties:expr),+ $(,)?), $($children:expr),+ $(,)?) => (
        $crate::dom::flex_node::col(($($properties),+), ($($children),+))
    );
    (style($($properties:expr),+ $(,)?), $children:expr) => (
        $crate::dom::flex_node::col(($($properties),+), ($children,))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::dom::flex_node::col((), ($($children),+))
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
