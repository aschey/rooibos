use next_tuple::NextTuple;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use reactive_graph::effect::RenderEffect;
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{
    AsDomNode, BlurEvent, ClickEvent, EventData, EventHandle, FocusEvent, KeyEvent, NodeId,
};
use tachys::prelude::*;

use super::dom_node::DomNode;
use super::layout::{
    AlignSelf, AspectRatio, Basis, Border, BorderBottom, BorderLeft, BorderRight, BorderTop,
    BorderX, BorderY, Clear, Disabled, Focusable, Grow, Height, Margin, MarginBottom, MarginLeft,
    MarginRight, MarginTop, MarginX, MarginY, MaxHeight, MaxWidth, MinHeight, MinWidth, Padding,
    PaddingBottom, PaddingLeft, PaddingRight, PaddingTop, PaddingX, PaddingY, Position, Property,
    Shrink, UpdateLayout, Width, align_self, aspect_ratio, basis, border, border_bottom,
    border_left, border_right, border_top, border_x, border_y, grow, height, margin, margin_bottom,
    margin_left, margin_right, margin_top, margin_x, margin_y, max_height, max_width, min_height,
    min_width, padding, padding_bottom, padding_left, padding_right, padding_top, padding_x,
    padding_y, position, shrink, width,
};
use crate::RooibosDom;

#[derive(Clone)]
pub struct DomWidget<P> {
    inner: DomNode,
    properties: P,
}

pub trait WidgetProperty: Property {}

impl WidgetProperty for () {}
impl WidgetProperty for Focusable {}
impl WidgetProperty for Clear {}
impl WidgetProperty for Disabled {}

pub struct DomWidgetNode(pub(crate) rooibos_dom::DomWidgetNode);

impl Render<RooibosDom> for DomWidgetNode {
    type State = RenderEffect<()>;

    fn build(self) -> Self::State {
        RenderEffect::new({
            let inner = self.0.clone();
            move |_| {
                inner.build();
            }
        })
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        *state = new;
    }
}

impl DomWidget<()> {
    pub fn new<T: 'static, F1: Fn() -> F2 + 'static, F2: FnMut(Rect, &mut Buffer) + 'static>(
        f: F1,
    ) -> Self {
        let dom_widget_node = rooibos_dom::DomWidgetNode::new::<T, _, _>(f);
        let inner = DomNode(rooibos_dom::DomNode::widget(dom_widget_node));
        Self {
            inner,
            properties: (),
        }
    }
}

impl<P> DomWidget<P> {
    pub fn new_with_properties<
        T: 'static,
        F1: Fn() -> F2 + 'static,
        F2: FnMut(Rect, &mut Buffer) + 'static,
    >(
        props: P,
        f: F1,
    ) -> Self {
        let dom_widget_node = rooibos_dom::DomWidgetNode::new::<T, _, _>(f);
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
}

impl<P> DomWidget<P>
where
    P: NextTuple,
{
    pub fn focusable<S>(self, focusable: S) -> DomWidget<P::Output<Focusable>>
    where
        S: Into<MaybeSignal<bool>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(Focusable(focusable.into())),
        }
    }

    pub fn disabled<S>(self, disabled: S) -> DomWidget<P::Output<Disabled>>
    where
        S: Into<MaybeSignal<bool>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(Disabled(disabled.into())),
        }
    }

    pub fn on_key_down<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_key_down(move |event, data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data, handle)
        });

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

    pub fn on_key_up<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_key_up(move |event, data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data, handle);
        });
        self
    }

    pub fn on_focus<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.inner.0 = self.inner.0.on_focus(move |event, data| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data);
        });
        self
    }

    pub fn on_blur<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.inner.0 = self.inner.0.on_blur(move |event, data| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data);
        });
        self
    }

    pub fn on_click<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(ClickEvent, EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_click(move |event, data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data, handle);
        });
        self
    }

    pub fn on_right_click<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(ClickEvent, EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_right_click(move |event, data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data, handle);
        });
        self
    }

    pub fn on_middle_click<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(ClickEvent, EventData, EventHandle) + 'static,
    {
        self.inner.0 = self.inner.0.on_middle_click(move |event, data, handle| {
            #[cfg(debug_assertions)]
            let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
            handler(event, data, handle);
        });
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

    pub border_left: BorderLeft,
    pub border_right: BorderRight,
    pub border_top: BorderTop,
    pub border_bottom: BorderBottom,
    pub border_x: BorderX,
    pub border_y: BorderY,
    pub border: Border,

    pub grow: Grow,
    pub shrink: Shrink,
    pub basis: Basis,
    pub align_self: AlignSelf,
}

impl WidgetProperty for LayoutProps {}

impl UpdateLayout for LayoutProps {
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

            border_left,
            border_right,
            border_top,
            border_bottom,
            border_x,
            border_y,
            border,
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

        border_left.update_layout(original_display, style);
        border_right.update_layout(original_display, style);
        border_top.update_layout(original_display, style);
        border_bottom.update_layout(original_display, style);
        border_x.update_layout(original_display, style);
        border_y.update_layout(original_display, style);
        border.update_layout(original_display, style);
    }
}

macro_rules! update_props {
    ($fn:ident, $inner:ty) => {
        fn $fn<S>(self, val: S) -> Self
        where
            S: Into<MaybeSignal<$inner>>,
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

    update_props!(border_left, taffy::LengthPercentage);
    update_props!(border_right, taffy::LengthPercentage);
    update_props!(border_top, taffy::LengthPercentage);
    update_props!(border_bottom, taffy::LengthPercentage);
    update_props!(border_x, taffy::LengthPercentage);
    update_props!(border_y, taffy::LengthPercentage);
    update_props!(border, taffy::LengthPercentage);

    update_props!(grow, f32);
    update_props!(shrink, f32);
    update_props!(align_self, taffy::AlignSelf);
    update_props!(basis, taffy::Dimension);
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
    ($struct_name:ident, $fn:ident, $inner:ty) => {
        impl WidgetProperty for $struct_name {}

        impl<P> DomWidget<P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> DomWidget<P::Output<$struct_name>>
            where
                S: Into<MaybeSignal<$inner>>,
            {
                DomWidget {
                    inner: self.inner,
                    properties: self.properties.next_tuple($fn(val).0),
                }
            }
        }

        impl UpdateLayoutPropsBorrowed for $struct_name {
            fn update_props(self, props: &mut LayoutProps) {
                props.$fn = self;
            }
        }

        impl LayoutProps {
            pub fn $fn<S>(mut self, val: S) -> Self
            where
                S: Into<MaybeSignal<$inner>>,
            {
                self.$fn = $fn(val).0;
                self
            }
        }
    };
}

widget_prop!(Width, width, taffy::Dimension);
widget_prop!(Height, height, taffy::Dimension);
widget_prop!(MinWidth, min_width, taffy::Dimension);
widget_prop!(MinHeight, min_height, taffy::Dimension);
widget_prop!(MaxWidth, max_width, taffy::Dimension);
widget_prop!(MaxHeight, max_height, taffy::Dimension);
widget_prop!(AspectRatio, aspect_ratio, f32);
widget_prop!(Position, position, taffy::Position);

widget_prop!(MarginLeft, margin_left, taffy::LengthPercentageAuto);
widget_prop!(MarginRight, margin_right, taffy::LengthPercentageAuto);
widget_prop!(MarginTop, margin_top, taffy::LengthPercentageAuto);
widget_prop!(MarginBottom, margin_bottom, taffy::LengthPercentageAuto);
widget_prop!(MarginX, margin_x, taffy::LengthPercentageAuto);
widget_prop!(MarginY, margin_y, taffy::LengthPercentageAuto);
widget_prop!(Margin, margin, taffy::LengthPercentageAuto);

widget_prop!(PaddingLeft, padding_left, taffy::LengthPercentage);
widget_prop!(PaddingRight, padding_right, taffy::LengthPercentage);
widget_prop!(PaddingTop, padding_top, taffy::LengthPercentage);
widget_prop!(PaddingBottom, padding_bottom, taffy::LengthPercentage);
widget_prop!(PaddingX, padding_x, taffy::LengthPercentage);
widget_prop!(PaddingY, padding_y, taffy::LengthPercentage);
widget_prop!(Padding, padding, taffy::LengthPercentage);

widget_prop!(BorderLeft, border_left, taffy::LengthPercentage);
widget_prop!(BorderRight, border_right, taffy::LengthPercentage);
widget_prop!(BorderTop, border_top, taffy::LengthPercentage);
widget_prop!(BorderBottom, border_bottom, taffy::LengthPercentage);
widget_prop!(BorderX, border_x, taffy::LengthPercentage);
widget_prop!(BorderY, border_y, taffy::LengthPercentage);
widget_prop!(Border, border, taffy::LengthPercentage);

widget_prop!(Grow, grow, f32);
widget_prop!(Shrink, shrink, f32);
widget_prop!(AlignSelf, align_self, taffy::AlignSelf);
widget_prop!(Basis, basis, taffy::Dimension);

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
