use std::sync::Arc;

use next_tuple::NextTuple;
use ratatui::layout::Rect;
use reactive_graph::effect::RenderEffect;
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::events::{
    BlurEvent, ClickHandler, EventData, EventHandle, FocusEvent, IntoClickHandler, IntoKeyHandler,
    KeyHandler,
};
use rooibos_dom::{AsDomNode, Borders, BuildNodeRenderer, NodeId};
use tachys::prelude::*;
use wasm_compat::sync::RwLock;

use super::dom_node::DomNode;
use super::layout::{
    AlignSelf, AspectRatio, Basis, BorderProp, Clear, Enabled, Focusable, Grow, Height, Margin,
    MarginBottom, MarginLeft, MarginRight, MarginTop, MarginX, MarginY, MaxHeight, MaxWidth,
    MinHeight, MinWidth, Padding, PaddingBottom, PaddingLeft, PaddingRight, PaddingTop, PaddingX,
    PaddingY, Position, Property, Shrink, UpdateLayout, Width, align_self, aspect_ratio, basis,
    borders, grow, height, margin, margin_bottom, margin_left, margin_right, margin_top, margin_x,
    margin_y, max_height, max_width, min_height, min_width, padding, padding_bottom, padding_left,
    padding_right, padding_top, padding_x, padding_y, position, shrink, width,
};
use crate::dom::RooibosDom;

#[derive(Clone, Debug)]
pub struct DomWidgetRef {
    inner: Arc<RwLock<DomNode>>,
}

#[derive(Clone)]
pub struct DomWidget<P> {
    inner: DomNode,
    properties: P,
}

pub trait WidgetProperty: Property {}

impl WidgetProperty for () {}
impl WidgetProperty for Focusable {}
impl WidgetProperty for Clear {}
impl WidgetProperty for Enabled {}

pub struct DomWidgetNode(pub(crate) rooibos_dom::DomWidgetNode);

impl Render<RooibosDom> for DomWidgetNode {
    type State = RenderEffect<()>;

    fn build(self) -> Self::State {
        RenderEffect::new({
            let inner = self.0.clone();
            move |_| {
                inner.build();
                inner.estimate_size();
            }
        })
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        *state = new;
    }
}

impl DomWidget<()> {
    pub fn new<T, R>(render_node: R) -> Self
    where
        T: 'static,
        R: BuildNodeRenderer + 'static,
    {
        let dom_widget_node = rooibos_dom::DomWidgetNode::new::<T, _>(render_node);
        let inner = DomNode(rooibos_dom::DomNode::widget(dom_widget_node));
        Self {
            inner,
            properties: (),
        }
    }

    pub fn get_ref() -> DomWidgetRef {
        DomWidgetRef {
            inner: Arc::new(RwLock::new(DomNode::default())),
        }
    }
}

impl<P> DomWidget<P> {
    pub fn new_with_properties<T, R>(props: P, render_node: R) -> Self
    where
        T: 'static,
        R: BuildNodeRenderer + 'static,
    {
        let dom_widget_node = rooibos_dom::DomWidgetNode::new::<T, _>(render_node);
        let inner = DomNode(rooibos_dom::DomNode::widget(dom_widget_node));
        Self {
            inner,
            properties: props,
        }
    }

    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.inner.0 = self.inner.0.id(id);
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.inner.0 = self.inner.0.class(class);
        self
    }

    pub fn z_index(mut self, z_index: i32) -> Self {
        self.inner.0 = self.inner.0.z_index(z_index);
        self
    }

    pub fn set_ref(&self, widget_ref: &mut DomWidgetRef) {
        *widget_ref.inner.write() = self.inner.clone();
    }
}

impl<P> DomWidget<P>
where
    P: NextTuple,
{
    pub fn focusable<S>(self, focusable: S) -> DomWidget<P::Output<Focusable>>
    where
        S: Into<Signal<bool>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(Focusable(focusable.into())),
        }
    }

    pub fn enabled<S>(self, enabled: S) -> DomWidget<P::Output<Enabled>>
    where
        S: Into<Signal<bool>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(Enabled(enabled.into())),
        }
    }

    pub fn on_key_down<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        let mut handler = handler.into_key_handler();
        self.inner.0 = self
            .inner
            .0
            .on_key_down(move |props| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler.handle(props)
            })
            .focusable(true);

        self
    }

    pub fn on_paste<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(String, EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_paste(move |val, data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(val, data, handle);
        });

        self
    }

    pub fn on_key_up<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        let mut handler = handler.into_key_handler();
        self.inner.0 = self
            .inner
            .0
            .on_key_up(move |props| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler.handle(props);
            })
            .focusable(true);
        self
    }

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

    pub fn on_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        let mut handler = handler.into_click_handler();
        self.inner.0 = self
            .inner
            .0
            .on_click(move |props| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler.handle(props);
            })
            .focusable(true);
        self
    }

    pub fn on_right_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        let mut handler = handler.into_click_handler();
        self.inner.0 = self
            .inner
            .0
            .on_right_click(move |props| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler.handle(props);
            })
            .focusable(true);
        self
    }

    pub fn on_middle_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        let mut handler = handler.into_click_handler();
        self.inner.0 = self
            .inner
            .0
            .on_middle_click(move |props| {
                #[cfg(debug_assertions)]
                let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                handler.handle(props);
            })
            .focusable(true);
        self
    }

    pub fn on_mouse_enter<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_mouse_enter(move |data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(data, handle);
        });
        self
    }

    pub fn on_mouse_leave<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_mouse_leave(move |data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(data, handle);
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

    pub fn layout_props(
        self,
        layout_props: LayoutProps,
    ) -> DomWidget<<P as NextTuple>::Output<LayoutProps>> {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(layout_props),
        }
    }
}

pub struct DomWidgetState<P>
where
    P: WidgetProperty,
{
    node: <DomNode as Render<RooibosDom>>::State,
    prop_state: <P as Property>::State,
}

impl<P> AsDomNode for DomWidgetState<P>
where
    P: WidgetProperty,
{
    fn as_dom_node(&self) -> &rooibos_dom::DomNode {
        self.node.as_dom_node()
    }
}

impl<P> Mountable<RooibosDom> for DomWidgetState<P>
where
    P: WidgetProperty,
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

impl<P> Render<RooibosDom> for DomWidget<P>
where
    P: WidgetProperty,
{
    type State = DomWidgetState<P>;

    fn build(self) -> Self::State {
        let inner_state = self.inner.build();
        let prop_state = self.properties.build(&inner_state.0);

        DomWidgetState {
            node: inner_state,
            prop_state,
        }
    }

    fn rebuild(mut self, state: &mut Self::State) {
        if self.inner == state.node.0 {
            self.inner.rebuild(&mut state.node);
            self.properties
                .rebuild(&state.node.0, &mut state.prop_state);
        } else {
            self.inner.replace_node(&state.node.0);
            *state = self.build();
        }
    }
}

pub trait UpdateLayoutPropsBorrowed {
    fn update_props(self, props: &mut LayoutProps);
}

pub struct LayoutPropsTuple<P>(P);

impl<P: NextTuple + WidgetProperty + UpdateLayoutPropsBorrowed> LayoutPropsTuple<P> {
    pub fn new(props: P) -> Self {
        Self(props)
    }

    pub fn to_props(self) -> LayoutProps {
        let mut props = LayoutProps::default();
        self.0.update_props(&mut props);
        props
    }
}

impl<P> NextTuple for LayoutPropsTuple<P>
where
    P: NextTuple + WidgetProperty + UpdateLayoutPropsBorrowed,
{
    type Output<Next> = P::Output<Next>;
    fn next_tuple<Next>(self, next: Next) -> Self::Output<Next> {
        self.0.next_tuple(next)
    }
}

#[derive(Default, Clone)]
pub struct LayoutProps {
    pub borders: BorderProp,
    pub simple: SimpleLayoutProps,
}

pub struct LayoutPropsState {
    borders: <BorderProp as Property>::State,
    simple: <SimpleLayoutProps as Property>::State,
}

impl Property for LayoutProps {
    type State = LayoutPropsState;

    fn build(self, node: &DomNode) -> Self::State {
        let borders = self.borders.build(node);
        let simple = self.simple.build(node);
        LayoutPropsState { borders, simple }
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        self.borders.rebuild(node, &mut state.borders);
        self.simple.rebuild(node, &mut state.simple);
    }
}

#[derive(Default, Clone)]
pub struct SimpleLayoutProps {
    pub width: Width,
    pub height: Height,
    pub min_width: MinWidth,
    pub min_height: MinHeight,
    pub max_width: MaxWidth,
    pub max_height: MaxHeight,
    pub aspect_ratio: AspectRatio,
    pub position: Position,

    pub margin_left: MarginLeft,
    pub margin_right: MarginRight,
    pub margin_top: MarginTop,
    pub margin_bottom: MarginBottom,
    pub margin_x: MarginX,
    pub margin_y: MarginY,
    pub margin: Margin,

    pub padding_left: PaddingLeft,
    pub padding_right: PaddingRight,
    pub padding_top: PaddingTop,
    pub padding_bottom: PaddingBottom,
    pub padding_x: PaddingX,
    pub padding_y: PaddingY,
    pub padding: Padding,

    pub grow: Grow,
    pub shrink: Shrink,
    pub basis: Basis,
    pub align_self: AlignSelf,
}

impl WidgetProperty for LayoutProps {}

impl UpdateLayout for SimpleLayoutProps {
    fn update_layout(&self, original_display: taffy::Display, style: &mut taffy::Style) {
        let Self {
            width,
            height,
            min_width,
            min_height,
            max_width,
            max_height,
            aspect_ratio,
            grow,
            shrink,
            basis,
            align_self,
            position,

            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            margin_x,
            margin_y,
            margin,

            padding_left,
            padding_right,
            padding_top,
            padding_bottom,
            padding_x,
            padding_y,
            padding,
        } = self;
        width.update_layout(original_display, style);
        height.update_layout(original_display, style);
        min_width.update_layout(original_display, style);
        min_height.update_layout(original_display, style);
        max_width.update_layout(original_display, style);
        max_height.update_layout(original_display, style);
        aspect_ratio.update_layout(original_display, style);
        grow.update_layout(original_display, style);
        shrink.update_layout(original_display, style);
        basis.update_layout(original_display, style);
        align_self.update_layout(original_display, style);
        position.update_layout(original_display, style);

        margin_left.update_layout(original_display, style);
        margin_right.update_layout(original_display, style);
        margin_top.update_layout(original_display, style);
        margin_bottom.update_layout(original_display, style);
        margin_x.update_layout(original_display, style);
        margin_y.update_layout(original_display, style);
        margin.update_layout(original_display, style);

        padding_left.update_layout(original_display, style);
        padding_right.update_layout(original_display, style);
        padding_top.update_layout(original_display, style);
        padding_bottom.update_layout(original_display, style);
        padding_x.update_layout(original_display, style);
        padding_y.update_layout(original_display, style);
        padding.update_layout(original_display, style);
    }
}

macro_rules! update_props {
    ($fn:ident, $inner:ty) => {
        fn $fn<S>(self, val: S) -> Self
        where
            S: Into<Signal<$inner>>,
        {
            let props = self.layout_props().$fn(val);
            self.update_props(props)
        }
    };
}

pub trait UpdateLayoutProps
where
    Self: Sized,
{
    fn layout_props(&self) -> LayoutProps;
    fn update_props(self, props: LayoutProps) -> Self;

    update_props!(width, taffy::Dimension);
    update_props!(height, taffy::Dimension);
    update_props!(min_width, taffy::Dimension);
    update_props!(min_height, taffy::Dimension);
    update_props!(max_width, taffy::Dimension);
    update_props!(max_height, taffy::Dimension);
    update_props!(aspect_ratio, f32);
    update_props!(position, taffy::Position);

    update_props!(margin_left, taffy::LengthPercentageAuto);
    update_props!(margin_right, taffy::LengthPercentageAuto);
    update_props!(margin_top, taffy::LengthPercentageAuto);
    update_props!(margin_bottom, taffy::LengthPercentageAuto);
    update_props!(margin_x, taffy::LengthPercentageAuto);
    update_props!(margin_y, taffy::LengthPercentageAuto);
    update_props!(margin, taffy::LengthPercentageAuto);

    update_props!(padding_left, taffy::LengthPercentage);
    update_props!(padding_right, taffy::LengthPercentage);
    update_props!(padding_top, taffy::LengthPercentage);
    update_props!(padding_bottom, taffy::LengthPercentage);
    update_props!(padding_x, taffy::LengthPercentage);
    update_props!(padding_y, taffy::LengthPercentage);
    update_props!(padding, taffy::LengthPercentage);

    update_props!(grow, f32);
    update_props!(shrink, f32);
    update_props!(align_self, taffy::AlignSelf);
    update_props!(basis, taffy::Dimension);

    update_props!(borders, Borders);
}

#[macro_export]
macro_rules! props {
    () => (
        $crate::LayoutPropsTuple::new(()).to_props()
    );
    ($($properties:expr),+ $(,)?) => (
        $crate::LayoutPropsTuple::new(($($properties),+)).to_props()
    );
}

macro_rules! widget_prop {
    ($struct_name:ident, $fn:ident, $inner:ty, $($path:ident).+) => {
        impl WidgetProperty for $struct_name {}

        impl<P> DomWidget<P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> DomWidget<P::Output<$struct_name>>
            where
                S: Into<Signal<$inner>>,
            {
                DomWidget {
                    inner: self.inner,
                    properties: self.properties.next_tuple($fn(val).0),
                }
            }
        }

        impl UpdateLayoutPropsBorrowed for $struct_name {
            fn update_props(self, props: &mut LayoutProps) {
                props.$($path).+ = self;
            }
        }

        impl LayoutProps {
            pub fn $fn<S>(mut self, val: S) -> Self
            where
                S: Into<Signal<$inner>>,
            {
                self.$($path).+ = $fn(val).0;
                self
            }
        }
    };
}

widget_prop!(Width, width, taffy::Dimension, simple.width);
widget_prop!(Height, height, taffy::Dimension, simple.height);
widget_prop!(MinWidth, min_width, taffy::Dimension, simple.min_width);
widget_prop!(MinHeight, min_height, taffy::Dimension, simple.min_height);
widget_prop!(MaxWidth, max_width, taffy::Dimension, simple.max_width);
widget_prop!(MaxHeight, max_height, taffy::Dimension, simple.max_height);
widget_prop!(AspectRatio, aspect_ratio, f32, simple.aspect_ratio);
widget_prop!(Position, position, taffy::Position, simple.position);

widget_prop!(
    MarginLeft,
    margin_left,
    taffy::LengthPercentageAuto,
    simple.margin_left
);
widget_prop!(
    MarginRight,
    margin_right,
    taffy::LengthPercentageAuto,
    simple.margin_right
);
widget_prop!(
    MarginTop,
    margin_top,
    taffy::LengthPercentageAuto,
    simple.margin_top
);
widget_prop!(
    MarginBottom,
    margin_bottom,
    taffy::LengthPercentageAuto,
    simple.margin_bottom
);
widget_prop!(
    MarginX,
    margin_x,
    taffy::LengthPercentageAuto,
    simple.margin_x
);
widget_prop!(
    MarginY,
    margin_y,
    taffy::LengthPercentageAuto,
    simple.margin_y
);
widget_prop!(Margin, margin, taffy::LengthPercentageAuto, simple.margin);

widget_prop!(
    PaddingLeft,
    padding_left,
    taffy::LengthPercentage,
    simple.padding_left
);
widget_prop!(
    PaddingRight,
    padding_right,
    taffy::LengthPercentage,
    simple.padding_right
);
widget_prop!(
    PaddingTop,
    padding_top,
    taffy::LengthPercentage,
    simple.padding_top
);
widget_prop!(
    PaddingBottom,
    padding_bottom,
    taffy::LengthPercentage,
    simple.padding_bottom
);
widget_prop!(
    PaddingX,
    padding_x,
    taffy::LengthPercentage,
    simple.padding_x
);
widget_prop!(
    PaddingY,
    padding_y,
    taffy::LengthPercentage,
    simple.padding_y
);
widget_prop!(Padding, padding, taffy::LengthPercentage, simple.padding);

widget_prop!(Grow, grow, f32, simple.grow);
widget_prop!(Shrink, shrink, f32, simple.shrink);
widget_prop!(AlignSelf, align_self, taffy::AlignSelf, simple.align_self);
widget_prop!(Basis, basis, taffy::Dimension, simple.basis);

widget_prop!(BorderProp, borders, Borders, borders);

macro_rules! impl_widget_property_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> WidgetProperty for ($($ty,)*)
            where $($ty: WidgetProperty,)*
        { }

        impl<$($ty,)*> UpdateLayoutPropsBorrowed for ($($ty,)*)
            where $($ty: UpdateLayoutPropsBorrowed,)*
        {
            fn update_props(self, props: &mut LayoutProps) {
                #[allow(non_snake_case)]
                let ($($ty,)*) = self;
                ($($ty.update_props(props),)*);
            }
        }
    }
}

impl_widget_property_for_tuples!(A);
impl_widget_property_for_tuples!(A, B);
impl_widget_property_for_tuples!(A, B, C);
impl_widget_property_for_tuples!(A, B, C, D);
impl_widget_property_for_tuples!(A, B, C, D, E);
impl_widget_property_for_tuples!(A, B, C, D, E, F);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
