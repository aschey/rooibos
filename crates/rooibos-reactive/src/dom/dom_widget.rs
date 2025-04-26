use std::sync::Arc;

use next_tuple::NextTuple;
use ratatui::layout::Rect;
use reactive_graph::effect::RenderEffect;
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::events::{
    BlurEvent, ClickHandler, DragHandler, EventData, EventHandle, FocusEvent, IntoClickHandler,
    IntoDragHandler, IntoKeyHandler, KeyHandler, NodeState,
};
use rooibos_dom::{AsDomNode, BuildNodeRenderer, NodeId};
use tachys::prelude::*;
use wasm_compat::sync::RwLock;

use super::dom_node::DomNode;
use super::layout::{
    AlignSelf, AspectRatio, Background, BorderProp, Borders, Class, Clear, Enabled, FlexBasis,
    FlexGrow, FlexShrink, Focusable, Height, Id, IntoAlignSelfSignal, IntoJustifySelfSignal,
    JustifySelf, Margin, MarginBottom, MarginLeft, MarginRight, MarginTop, MarginX, MarginY,
    MaxHeight, MaxWidth, MinHeight, MinWidth, Overflow, OverflowX, OverflowY, Padding,
    PaddingBottom, PaddingLeft, PaddingRight, PaddingTop, PaddingX, PaddingY, Position, Property,
    UpdateLayout, Width, ZIndex, align_self, aspect_ratio, background, borders, class, clear,
    enabled, flex_basis, flex_grow, flex_shrink, focusable, height, id, justify_self, margin,
    margin_bottom, margin_left, margin_right, margin_top, margin_x, margin_y, max_height,
    max_width, min_height, min_width, overflow, overflow_x, overflow_y, padding, padding_bottom,
    padding_left, padding_right, padding_top, padding_x, padding_y, position, width, z_index,
};
#[cfg(feature = "effects")]
use super::layout::{Effect, effect};
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

    pub fn set_ref(&self, widget_ref: &mut DomWidgetRef) {
        *widget_ref.inner.write() = self.inner.clone();
    }
}

impl<P> DomWidget<P>
where
    P: NextTuple,
{
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

    pub fn on_state_change<F>(mut self, mut handler: F) -> Self
    where
        F: FnMut(NodeState, EventData) + 'static,
    {
        self.inner.0 = self
            .inner
            .0
            .on_state_change(move |event, data| {
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

    pub fn on_mouse_drag<H>(mut self, handler: H) -> Self
    where
        H: IntoDragHandler + 'static,
    {
        let mut handler = handler.into_drag_handler();
        self.inner.0 = self
            .inner
            .0
            .on_mouse_drag(move |props| {
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
    P: WidgetProperty + 'static,
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
    pub simple: SimpleLayoutProps,
    pub borders: BorderProp,
    pub background: Background,
    pub focusable: Focusable,
    pub clear: Clear,
    pub enabled: Enabled,
    pub id: Id,
    pub class: Class,
    pub z_index: ZIndex,
    #[cfg(feature = "effects")]
    pub effect: Effect,
}

pub struct LayoutPropsState {
    simple: <SimpleLayoutProps as Property>::State,
    borders: <BorderProp as Property>::State,
    background: <Background as Property>::State,
    focusable: <Focusable as Property>::State,
    clear: <Clear as Property>::State,
    enabled: <Enabled as Property>::State,
    id: <Id as Property>::State,
    class: <Class as Property>::State,
    z_index: <ZIndex as Property>::State,
    #[cfg(feature = "effects")]
    effect: <Effect as Property>::State,
}

macro_rules! build_props {
    ($self:ident, $node:ident, $($prop:ident),*) => {
        $(let $prop = $self.$prop.build($node);)*
    };
}

macro_rules! rebuild_props {
    ($self:ident, $node:ident, $state:ident, $($prop:ident),*) => {
        $($self.$prop.rebuild($node, &mut $state.$prop);)*
    };
}

impl Property for LayoutProps {
    type State = LayoutPropsState;

    fn build(self, node: &DomNode) -> Self::State {
        build_props!(
            self, node, borders, background, focusable, simple, clear, enabled, id, class, z_index
        );
        #[cfg(feature = "effects")]
        build_props!(self, node, effect);

        LayoutPropsState {
            borders,
            background,
            focusable,
            simple,
            clear,
            enabled,
            id,
            class,
            z_index,
            #[cfg(feature = "effects")]
            effect,
        }
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        rebuild_props!(
            self, node, state, borders, background, focusable, simple, clear, enabled, id, class,
            z_index
        );
        #[cfg(feature = "effects")]
        rebuild_props!(self, node, state, effect);
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

    pub overflow: Overflow,
    pub overflow_x: OverflowX,
    pub overflow_y: OverflowY,

    pub flex_grow: FlexGrow,
    pub flex_shrink: FlexShrink,
    pub flex_basis: FlexBasis,
    pub align_self: AlignSelf,
    pub justify_self: JustifySelf,
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
            flex_grow,
            flex_shrink,
            flex_basis,
            align_self,
            justify_self,
            position,

            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            margin_x,
            margin_y,
            margin,

            overflow_x,
            overflow_y,
            overflow,

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
        flex_grow.update_layout(original_display, style);
        flex_shrink.update_layout(original_display, style);
        flex_basis.update_layout(original_display, style);
        align_self.update_layout(original_display, style);
        justify_self.update_layout(original_display, style);
        position.update_layout(original_display, style);

        margin_left.update_layout(original_display, style);
        margin_right.update_layout(original_display, style);
        margin_top.update_layout(original_display, style);
        margin_bottom.update_layout(original_display, style);
        margin_x.update_layout(original_display, style);
        margin_y.update_layout(original_display, style);
        margin.update_layout(original_display, style);

        overflow_x.update_layout(original_display, style);
        overflow_y.update_layout(original_display, style);
        overflow.update_layout(original_display, style);

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

macro_rules! update_custom_props {
    ($fn:ident, $bound:path) => {
        fn $fn<S>(self, val: S) -> Self
        where
            S: $bound,
        {
            let props = self.layout_props().$fn(val);
            self.update_props(props)
        }
    };
}

macro_rules! update_dimension_props {
    ($fn:ident) => {
        update_custom_props!($fn, $crate::dom::layout::IntoDimensionSignal);
    };
}

pub trait UpdateLayoutProps
where
    Self: Sized,
{
    fn layout_props(&self) -> LayoutProps;
    fn update_props(self, props: LayoutProps) -> Self;

    update_dimension_props!(width);
    update_dimension_props!(height);
    update_dimension_props!(min_width);
    update_dimension_props!(min_height);
    update_dimension_props!(max_width);
    update_dimension_props!(max_height);
    update_props!(aspect_ratio, f32);
    update_props!(position, taffy::Position);

    update_dimension_props!(margin_left);
    update_dimension_props!(margin_right);
    update_dimension_props!(margin_top);
    update_dimension_props!(margin_bottom);
    update_dimension_props!(margin_x);
    update_dimension_props!(margin_y);
    update_dimension_props!(margin);

    update_dimension_props!(padding_left);
    update_dimension_props!(padding_right);
    update_dimension_props!(padding_top);
    update_dimension_props!(padding_bottom);
    update_dimension_props!(padding_x);
    update_dimension_props!(padding_y);
    update_dimension_props!(padding);

    update_props!(overflow, taffy::Overflow);
    update_props!(overflow_x, taffy::Overflow);
    update_props!(overflow_y, taffy::Overflow);

    update_props!(flex_grow, f32);
    update_props!(flex_shrink, f32);
    update_custom_props!(align_self, IntoAlignSelfSignal);
    update_custom_props!(justify_self, IntoJustifySelfSignal);
    update_dimension_props!(flex_basis);

    update_props!(borders, Borders);
    update_props!(background, ratatui::style::Color);
    update_props!(focusable, bool);
    update_props!(clear, bool);
    update_props!(enabled, bool);
    update_props!(class, Vec<String>);
    update_props!(z_index, i32);
    #[cfg(feature = "effects")]
    update_props!(effect, super::layout::SyncEffect);

    fn id<S>(self, val: S) -> Self
    where
        S: Into<NodeId>,
    {
        let props = self.layout_props().id(val);
        self.update_props(props)
    }
}

impl WidgetProperty for Id {}

impl<P> DomWidget<P>
where
    P: NextTuple,
{
    pub fn id<S>(self, val: S) -> DomWidget<P::Output<Id>>
    where
        S: Into<NodeId>,
    {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(id(val)),
        }
    }
}

impl UpdateLayoutPropsBorrowed for Id {
    fn update_props(self, props: &mut LayoutProps) {
        props.id = self;
    }
}

impl LayoutProps {
    pub fn id<S>(mut self, val: S) -> Self
    where
        S: Into<NodeId>,
    {
        self.id = id(val);
        self
    }
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
                    properties: self.properties.next_tuple($fn(val)),
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
                self.$($path).+ = $fn(val);
                self
            }
        }
    };
}

macro_rules! custom_widget_prop {
    ($struct_name:ident, $fn:ident, $bound:path, $($path:ident).+) => {
        impl WidgetProperty for $struct_name {}

        impl<P> DomWidget<P>
        where
            P: NextTuple,
        {
            pub fn $fn<S>(self, val: S) -> DomWidget<P::Output<$struct_name>>
            where
                S: $bound,
            {
                DomWidget {
                    inner: self.inner,
                    properties: self.properties.next_tuple($fn(val)),
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
                S: $bound,
            {
                self.$($path).+ = $fn(val);
                self
            }
        }
    };
}

macro_rules! dimension_widget_prop {
    ($struct_name:ident, $fn:ident, $($path:tt)+) => {
        custom_widget_prop!($struct_name, $fn, $crate::dom::layout::IntoDimensionSignal, $($path)+);
    }
}

dimension_widget_prop!(Width, width, simple.width);
dimension_widget_prop!(Height, height, simple.height);
dimension_widget_prop!(MinWidth, min_width, simple.min_width);
dimension_widget_prop!(MinHeight, min_height, simple.min_height);
dimension_widget_prop!(MaxWidth, max_width, simple.max_width);
dimension_widget_prop!(MaxHeight, max_height, simple.max_height);
widget_prop!(AspectRatio, aspect_ratio, f32, simple.aspect_ratio);
widget_prop!(Position, position, taffy::Position, simple.position);

dimension_widget_prop!(MarginLeft, margin_left, simple.margin_left);
dimension_widget_prop!(MarginRight, margin_right, simple.margin_right);
dimension_widget_prop!(MarginTop, margin_top, simple.margin_top);
dimension_widget_prop!(MarginBottom, margin_bottom, simple.margin_bottom);
dimension_widget_prop!(MarginX, margin_x, simple.margin_x);
dimension_widget_prop!(MarginY, margin_y, simple.margin_y);
dimension_widget_prop!(Margin, margin, simple.margin);

widget_prop!(Overflow, overflow, taffy::Overflow, simple.overflow);
widget_prop!(OverflowX, overflow_x, taffy::Overflow, simple.overflow_x);
widget_prop!(OverflowY, overflow_y, taffy::Overflow, simple.overflow_y);

dimension_widget_prop!(PaddingLeft, padding_left, simple.padding_left);
dimension_widget_prop!(PaddingRight, padding_right, simple.padding_right);
dimension_widget_prop!(PaddingTop, padding_top, simple.padding_top);
dimension_widget_prop!(PaddingBottom, padding_bottom, simple.padding_bottom);
dimension_widget_prop!(PaddingX, padding_x, simple.padding_x);
dimension_widget_prop!(PaddingY, padding_y, simple.padding_y);
dimension_widget_prop!(Padding, padding, simple.padding);

widget_prop!(FlexGrow, flex_grow, f32, simple.flex_grow);
widget_prop!(FlexShrink, flex_shrink, f32, simple.flex_shrink);
custom_widget_prop!(
    AlignSelf,
    align_self,
    IntoAlignSelfSignal,
    simple.align_self
);
custom_widget_prop!(
    JustifySelf,
    justify_self,
    IntoJustifySelfSignal,
    simple.justify_self
);
dimension_widget_prop!(FlexBasis, flex_basis, simple.flex_basis);

widget_prop!(BorderProp, borders, Borders, borders);
widget_prop!(Background, background, ratatui::style::Color, background);
widget_prop!(Focusable, focusable, bool, focusable);
widget_prop!(Clear, clear, bool, clear);
widget_prop!(Enabled, enabled, bool, enabled);
widget_prop!(Class, class, Vec<String>, class);
widget_prop!(ZIndex, z_index, i32, z_index);
#[cfg(feature = "effects")]
widget_prop!(Effect, effect, super::layout::SyncEffect, effect);

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
