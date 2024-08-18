use std::any::type_name;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use next_tuple::NextTuple;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use reactive_graph::effect::RenderEffect;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::*;
use terminput::{KeyEvent, MouseEvent};

use super::dom_node::{DomNode, NodeId};
use super::layout::{
    align_self, aspect_ratio, basis, border, border_bottom, border_left, border_right, border_top,
    border_x, border_y, grow, height, margin, margin_bottom, margin_left, margin_right, margin_top,
    margin_x, margin_y, max_height, max_width, min_height, min_width, padding, padding_bottom,
    padding_left, padding_right, padding_top, padding_x, padding_y, shrink, width, AlignSelf,
    AspectRatio, Basis, Border, BorderBottom, BorderLeft, BorderRight, BorderTop, BorderX, BorderY,
    Grow, Height, Margin, MarginBottom, MarginLeft, MarginRight, MarginTop, MarginX, MarginY,
    MaxHeight, MaxWidth, MinHeight, MinWidth, Padding, PaddingBottom, PaddingLeft, PaddingRight,
    PaddingTop, PaddingX, PaddingY, Shrink, UpdateLayout, Width,
};
use super::{AsDomNode, Constraint, Focusable, Property};
use crate::widgets::WidgetRole;
use crate::{
    next_node_id, refresh_dom, BlurEvent, Constrainable, EventData, FocusEvent, Role, RooibosDom,
};

pub(crate) type DomWidgetFn = Box<dyn FnMut(Rect, &mut Buffer)>;

#[derive(Clone)]
pub struct DomWidget<P> {
    inner: DomNode,
    properties: P,
}

pub trait WidgetProperty: Property {}

impl WidgetProperty for () {}
impl WidgetProperty for Constraint {}
impl WidgetProperty for Focusable {}

#[derive(Clone)]
pub struct DomWidgetNode {
    f: Rc<dyn Fn() -> DomWidgetFn>,
    rc_f: Rc<RefCell<DomWidgetFn>>,
    id: u32,
    pub(crate) widget_type: String,
    pub(crate) role: Option<Role>,
}

impl PartialEq for DomWidgetNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomWidgetNode {}

impl Debug for DomWidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}/>", self.widget_type)
    }
}

impl DomWidgetNode {
    pub fn new<T: 'static, F1: Fn() -> F2 + 'static, F2: FnMut(Rect, &mut Buffer) + 'static>(
        f: F1,
    ) -> Self {
        let widget_type = type_name::<T>();
        let role = T::widget_role();
        let id = next_node_id();
        let rc_f: Rc<RefCell<DomWidgetFn>> = Rc::new(RefCell::new(Box::new(|_, _| {})));

        Self {
            id,
            role,
            rc_f,
            f: Rc::new(move || Box::new((f)())),
            widget_type: widget_type.into(),
        }
    }

    pub(crate) fn render(&self, rect: Rect, buf: &mut Buffer) {
        (*self.rc_f).borrow_mut()(rect, buf);
    }
}

impl Render<RooibosDom> for DomWidgetNode {
    type State = RenderEffect<()>;

    fn build(self) -> Self::State {
        RenderEffect::new({
            let f = self.f.clone();
            let rc_f = self.rc_f.clone();
            move |_| {
                (*rc_f.borrow_mut()) = (f)();
                refresh_dom();
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
        let dom_widget_node = DomWidgetNode::new::<T, _, _>(f);
        let inner = DomNode::widget(dom_widget_node);
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
        let dom_widget_node = DomWidgetNode::new::<T, _, _>(f);
        let inner = DomNode::widget(dom_widget_node);
        Self {
            inner,
            properties: props,
        }
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn class(self, class: impl Into<String>) -> Self {
        self.inner.set_class(class);
        self
    }

    pub fn z_index(self, z_index: i32) -> Self {
        self.inner.set_z_index(z_index);
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

    pub fn on_key_down<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_key_down(handler));

        this
    }

    pub fn on_paste<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(String, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_paste(handler));

        this
    }

    pub fn on_key_up<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_key_up(handler));
        this
    }

    pub fn on_focus<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_focus(handler));
        this
    }

    pub fn on_blur<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_blur(handler));
        this
    }

    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: FnMut(MouseEvent, EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_click(handler));
        self
    }

    pub fn on_mouse_enter<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_mouse_enter(handler));
        self
    }

    pub fn on_mouse_leave<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_mouse_leave(handler));
        self
    }

    pub fn on_size_change<F>(self, handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_size_change(handler));
        self
    }

    pub fn layout_props(
        self,
        layout_props: LayoutProps,
    ) -> DomWidget<<P as NextTuple>::Output<(LayoutProps,)>> {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple((layout_props,)),
        }
    }
}

impl<P> Constrainable for DomWidget<P>
where
    P: NextTuple,
{
    type Output = DomWidget<P::Output<super::Constraint>>;

    fn constraint<S>(self, constraint: S) -> Self::Output
    where
        S: Into<MaybeSignal<ratatui::layout::Constraint>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self
                .properties
                .next_tuple(super::Constraint(constraint.into())),
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
    fn as_dom_node(&self) -> &DomNode {
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
            refresh_dom();
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
    width: Width,
    height: Height,
    min_width: MinWidth,
    min_height: MinHeight,
    max_width: MaxWidth,
    max_height: MaxHeight,
    aspect_ratio: AspectRatio,

    margin_left: MarginLeft,
    margin_right: MarginRight,
    margin_top: MarginTop,
    margin_bottom: MarginBottom,
    margin_x: MarginX,
    margin_y: MarginY,
    margin: Margin,

    padding_left: PaddingLeft,
    padding_right: PaddingRight,
    padding_top: PaddingTop,
    padding_bottom: PaddingBottom,
    padding_x: PaddingX,
    padding_y: PaddingY,
    padding: Padding,

    border_left: BorderLeft,
    border_right: BorderRight,
    border_top: BorderTop,
    border_bottom: BorderBottom,
    border_x: BorderX,
    border_y: BorderY,
    border: Border,

    grow: Grow,
    shrink: Shrink,
    basis: Basis,
    align_self: AlignSelf,
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
