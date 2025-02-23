use next_tuple::NextTuple;
use reactive_graph::computed::Memo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::NodeId;
pub use rooibos_dom::{BorderType, Borders};
use taffy::Display;
use taffy::prelude::{TaffyAuto, TaffyZero};

use super::{DomNode, with_nodes_mut};
use crate::derive_signal;

pub trait Property {
    type State;

    fn build(self, node: &DomNode) -> Self::State;

    fn rebuild(self, node: &DomNode, state: &mut Self::State);
}

impl Property for () {
    type State = ();

    fn build(self, _node: &DomNode) -> Self::State {}

    fn rebuild(self, _node: &DomNode, _state: &mut Self::State) {}
}

pub(crate) trait UpdateLayout {
    fn update_layout(&self, original_display: taffy::Display, style: &mut taffy::Style);
}

macro_rules! impl_next_tuple {
    ($struct:ident) => {
        impl NextTuple for $struct {
            type Output<Next> = <($struct,) as NextTuple>::Output<Next>;
            fn next_tuple<Next>(self, next: Next) -> Self::Output<Next> {
                (self,).next_tuple(next)
            }
        }
    };
}

macro_rules! signal_wrapper {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr) => {
        #[derive(Default, Clone)]
        pub struct $struct_name(pub(crate) Option<Signal<$inner>>);

        impl_next_tuple!($struct_name);

        impl From<Signal<$inner>> for $struct_name {
            fn from(val: Signal<$inner>) -> Self {
                $struct_name(Some(val))
            }
        }

        impl From<$struct_name> for Signal<$inner> {
            fn from(val: $struct_name) -> Self {
                val.0.unwrap_or_else(|| $default.into())
            }
        }

        impl $struct_name {
            pub fn value(&self) -> Option<Signal<$inner>> {
                self.0.clone()
            }
        }

        pub fn $fn(val: impl Into<Signal<$inner>>) -> $struct_name {
            $struct_name(Some(val.into()))
        }
    };
}

pub fn chars(val: impl Into<Signal<u32>>) -> Signal<taffy::Dimension> {
    let val = val.into();
    derive_signal!(taffy::Dimension::Length(val.get() as f32))
}

pub fn pct(val: impl Into<Signal<u32>>) -> Signal<taffy::Dimension> {
    let val = val.into();
    derive_signal!(taffy::Dimension::Percent(val.get() as f32 / 100.0))
}

pub fn length_percentage_pct(val: impl Into<Signal<u32>>) -> Signal<taffy::LengthPercentage> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentage::Percent(val.get() as f32 / 100.0))
}

pub fn length_percentage_chars(val: impl Into<Signal<u32>>) -> Signal<taffy::LengthPercentage> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentage::Length(val.get() as f32))
}

pub fn length_percentage_auto_pct(
    val: impl Into<Signal<u32>>,
) -> Signal<taffy::LengthPercentageAuto> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentageAuto::Percent(
        val.get() as f32 / 100.0
    ))
}

pub fn length_percentage_auto_chars(
    val: impl Into<Signal<u32>>,
) -> Signal<taffy::LengthPercentageAuto> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentageAuto::Length(val.get() as f32))
}

signal_wrapper!(Show, show, bool, true);

impl Property for Show {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                let original_display = *nodes.original_display(key);
                let enabled = self.0.get().unwrap_or(true);
                nodes.set_enabled(key, enabled);
                nodes.update_layout(key, |s| {
                    s.display = if enabled {
                        original_display
                    } else {
                        Display::None
                    }
                })
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(BorderProp, borders, Borders, Borders::default());

impl Property for BorderProp {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            let Some(border) = &self.0 else {
                return;
            };
            let border = border.get();
            let rect = border.to_rect();
            let block = border.into_block();

            with_nodes_mut(|nodes| {
                nodes.set_block(key, block);
                nodes.update_layout(key, |s| s.border = rect);
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(ZIndex, z_index, i32, 0);

impl Property for ZIndex {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        let z_index = self.0.map(|v| Memo::new(move |_| v.get()));
        RenderEffect::new(move |_| {
            if let Some(z_index) = z_index {
                with_nodes_mut(|nodes| {
                    nodes.set_z_index(key, z_index.get());
                });
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

#[cfg(feature = "effects")]
signal_wrapper!(
    Effect,
    effect,
    rooibos_dom::tachyonfx::Effect,
    rooibos_dom::tachyonfx::Effect::new(rooibos_dom::tachyonfx::fx::sequence(&[]))
);

#[cfg(feature = "effects")]
impl Property for Effect {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            if let Some(effect) = self.0 {
                with_nodes_mut(|nodes| {
                    nodes.set_effect(key, effect.get());
                });
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(Clear, clear, bool, false);

impl Property for Clear {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            if let Some(clear) = self.0 {
                with_nodes_mut(|nodes| {
                    nodes.set_clear(key, clear.get());
                });
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(Class, class, Vec<String>, Vec::default());

impl Property for Class {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            if let Some(class) = self.0 {
                with_nodes_mut(|nodes| {
                    nodes.set_class(key, class.get());
                });
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

#[derive(Default, Clone)]
pub struct Id(pub(crate) Option<NodeId>);

impl_next_tuple!(Id);

impl Id {
    pub fn value(&self) -> Option<NodeId> {
        self.0.clone()
    }
}

pub fn id(id: impl Into<NodeId>) -> Id {
    Id(Some(id.into()))
}

impl Property for Id {
    type State = ();

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        if let Some(id) = self.0 {
            with_nodes_mut(|nodes| {
                nodes.set_id(key, id);
            });
        }
    }

    fn rebuild(self, _node: &DomNode, _state: &mut Self::State) {}
}

impl<T> Property for T
where
    T: UpdateLayout + 'static,
{
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                let original_display = *nodes.original_display(key);
                nodes.update_layout(key, |s| self.update_layout(original_display, s))
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(Focusable, focusable, bool, false);

impl Property for Focusable {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            if let Some(focusable) = self.0 {
                with_nodes_mut(|nodes| {
                    nodes.set_focusable(key, focusable.get());
                });
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(Enabled, enabled, bool, true);

impl Property for Enabled {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        let enabled = self.0.map(|v| Memo::new(move |_| v.get()));
        RenderEffect::new(move |_| {
            if let Some(enabled) = enabled {
                with_nodes_mut(|nodes| {
                    nodes.set_enabled(key, enabled.get());
                });
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

#[macro_export]
macro_rules! width {
    ($val:tt %) => {
        $crate::dom::layout::width($crate::dom::layout::pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::width($crate::dom::layout::chars($val))
    };
}

#[macro_export]
macro_rules! height {
    ($val:tt %) => {
        $crate::dom::layout::height($crate::dom::layout::pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::height($crate::dom::layout::chars($val))
    };
}

#[macro_export]
macro_rules! min_width {
    ($val:tt %) => {
        $crate::dom::layout::min_width($crate::dom::layout::pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::min_width($crate::dom::layout::chars($val))
    };
}

#[macro_export]
macro_rules! min_height {
    ($val:tt %) => {
        $crate::dom::layout::min_height($crate::dom::layout::pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::min_height($crate::dom::layout::chars($val))
    };
}

#[macro_export]
macro_rules! max_width {
    ($val:tt %) => {
        $crate::dom::layout::max_width($crate::dom::layout::pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::max_width($crate::dom::layout::chars($val))
    };
}

#[macro_export]
macro_rules! max_height {
    ($val:tt %) => {
        $crate::dom::layout::max_height($crate::dom::layout::pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::max_height($crate::dom::layout::chars($val))
    };
}

#[macro_export]
macro_rules! padding_left {
    ($val:tt %) => {
        $crate::dom::layout::padding_left($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding_left($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_right {
    ($val:tt %) => {
        $crate::dom::layout::padding_right($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding_right($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_top {
    ($val:tt %) => {
        $crate::dom::layout::padding_top($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding_top($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_bottom {
    ($val:tt %) => {
        $crate::dom::layout::padding_bottom($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding_bottom($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_x {
    ($val:tt %) => {
        $crate::dom::layout::padding_x($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding_x($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_y {
    ($val:tt %) => {
        $crate::dom::layout::padding_y($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding_y($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding {
    ($val:tt %) => {
        $crate::dom::layout::padding($crate::dom::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::padding($crate::dom::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! margin_left {
    ($val:tt %) => {
        $crate::dom::layout::margin_left($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin_left($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

#[macro_export]
macro_rules! margin_right {
    ($val:tt %) => {
        $crate::dom::layout::margin_right($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin_right($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

#[macro_export]
macro_rules! margin_top {
    ($val:tt %) => {
        $crate::dom::layout::margin_top($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin_top($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

#[macro_export]
macro_rules! margin_bottom {
    ($val:tt %) => {
        $crate::dom::layout::margin_bottom($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin_bottom($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

#[macro_export]
macro_rules! margin_x {
    ($val:tt %) => {
        $crate::dom::layout::margin_x($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin_x($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

#[macro_export]
macro_rules! margin_y {
    ($val:tt %) => {
        $crate::dom::layout::margin_y($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin_y($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

#[macro_export]
macro_rules! margin {
    ($val:tt %) => {
        $crate::dom::layout::margin($crate::dom::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::dom::layout::margin($crate::dom::layout::length_percentage_auto_chars($val))
    };
}

macro_rules! layout_prop {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr, $($($props:ident).+),+) => {
        signal_wrapper!($struct_name, $fn, $inner, $default);

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    $(style.$($props).* = inner.get();)+
                }

            }
        }
    };
}

macro_rules! layout_prop_opt {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr, $($prop:ident).*) => {
        #[derive(Default, Clone)]
        pub struct $struct_name(pub(crate) Option<Signal<$inner>>);

        impl From<Signal<$inner>> for $struct_name {
            fn from(val: Signal<$inner>) -> Self {
                $struct_name(Some(val))
            }
        }

        impl From<$struct_name> for Signal<$inner> {
            fn from(val: $struct_name) -> Self {
                val.0.unwrap_or_else(|| $default.into())
            }
        }

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    style.$($prop).* = Some(inner.get());
                }
            }
        }

        impl_next_tuple!($struct_name);

        pub fn $fn(val: impl Into<Signal<$inner>>) -> $struct_name {
            $struct_name(Some(val.into()))
        }
    };
}

// Generic properties
layout_prop!(
    Width,
    width,
    taffy::Dimension,
    taffy::Dimension::Auto,
    size.width
);
layout_prop!(
    Height,
    height,
    taffy::Dimension,
    taffy::Dimension::Auto,
    size.height
);
layout_prop!(
    MinWidth,
    min_width,
    taffy::Dimension,
    taffy::Dimension::Auto,
    min_size.width
);
layout_prop!(
    MinHeight,
    min_height,
    taffy::Dimension,
    taffy::Dimension::Auto,
    min_size.height
);
layout_prop!(
    MaxWidth,
    max_width,
    taffy::Dimension,
    taffy::Dimension::Auto,
    max_size.width
);
layout_prop!(
    MaxHeight,
    max_height,
    taffy::Dimension,
    taffy::Dimension::Auto,
    max_size.height
);
layout_prop_opt!(AspectRatio, aspect_ratio, f32, 0.0, aspect_ratio);
layout_prop!(
    Position,
    position,
    taffy::style::Position,
    taffy::style::Position::default(),
    position
);

layout_prop!(
    MarginLeft,
    margin_left,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.left
);
layout_prop!(
    MarginRight,
    margin_right,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.right
);
layout_prop!(
    MarginTop,
    margin_top,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.top
);
layout_prop!(
    MarginBottom,
    margin_bottom,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.bottom
);
layout_prop!(
    MarginX,
    margin_x,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.left,
    margin.right
);
layout_prop!(
    MarginY,
    margin_y,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.top,
    margin.bottom
);
layout_prop!(
    Margin,
    margin,
    taffy::LengthPercentageAuto,
    taffy::LengthPercentageAuto::Auto,
    margin.top,
    margin.bottom,
    margin.left,
    margin.right
);

layout_prop!(
    PaddingLeft,
    padding_left,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.left
);
layout_prop!(
    PaddingRight,
    padding_right,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.right
);
layout_prop!(
    PaddingTop,
    padding_top,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.top
);
layout_prop!(
    PaddingBottom,
    padding_bottom,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.bottom
);
layout_prop!(
    PaddingX,
    padding_x,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.left,
    padding.right
);
layout_prop!(
    PaddingY,
    padding_y,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.top,
    padding.bottom
);
layout_prop!(
    Padding,
    padding,
    taffy::LengthPercentage,
    taffy::LengthPercentage::ZERO,
    padding.top,
    padding.bottom,
    padding.left,
    padding.right
);

layout_prop!(
    OverflowX,
    overflow_x,
    taffy::Overflow,
    taffy::Overflow::default(),
    overflow.x
);
layout_prop!(
    OverflowY,
    overflow_y,
    taffy::Overflow,
    taffy::Overflow::default(),
    overflow.y
);
layout_prop!(
    Overflow,
    overflow,
    taffy::Overflow,
    taffy::Overflow::default(),
    overflow.x,
    overflow.y
);

// Flex properties
layout_prop!(
    Wrap,
    wrap,
    taffy::FlexWrap,
    taffy::FlexWrap::default(),
    flex_wrap
);
layout_prop_opt!(
    AlignItems,
    align_items,
    taffy::AlignItems,
    taffy::AlignItems::Start,
    align_items
);
layout_prop_opt!(
    AlignContent,
    align_content,
    taffy::AlignContent,
    taffy::AlignContent::Start,
    align_content
);
layout_prop_opt!(
    JustifyContent,
    justify_content,
    taffy::JustifyContent,
    taffy::JustifyContent::Start,
    justify_content
);
layout_prop!(
    Gap,
    gap,
    taffy::Size<taffy::LengthPercentage>,
    taffy::Size::zero(),
    gap
);
layout_prop!(Grow, grow, f32, 0.0, flex_grow);
layout_prop!(Shrink, shrink, f32, 0.0, flex_shrink);
layout_prop_opt!(
    AlignSelf,
    align_self,
    taffy::AlignSelf,
    taffy::AlignSelf::Start,
    align_self
);
layout_prop!(
    Basis,
    basis,
    taffy::Dimension,
    taffy::Dimension::AUTO,
    flex_basis
);

macro_rules! impl_property_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> Property for ($($ty,)*)
            where $($ty: Property,)*
        {
            type State = ($($ty::State,)*);

            fn build(self, element: &DomNode) -> Self::State {
                #[allow(non_snake_case)]
                let ($($ty,)*) = self;
                ($($ty.build(element),)*)
            }

            fn rebuild(self, element: &DomNode, state: &mut Self::State) {
                paste::paste! {
                    #[allow(non_snake_case)]
                    let ($($ty,)*) = self;
                    #[allow(non_snake_case)]
                    let ($([<state_ $ty:lower>],)*) = state;
                    $($ty.rebuild(element, [<state_ $ty:lower>]));*
                }
            }
        }
    }
}

impl_property_for_tuples!(A);
impl_property_for_tuples!(A, B);
impl_property_for_tuples!(A, B, C);
impl_property_for_tuples!(A, B, C, D);
impl_property_for_tuples!(A, B, C, D, E);
impl_property_for_tuples!(A, B, C, D, E, F);
impl_property_for_tuples!(A, B, C, D, E, F, G);
impl_property_for_tuples!(A, B, C, D, E, F, G, H);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
