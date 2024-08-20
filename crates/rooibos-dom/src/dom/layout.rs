use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeSignal, Signal};
use taffy::Display;

use super::{with_nodes_mut, DomNode};
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

pub fn chars(val: impl Into<MaybeSignal<f32>>) -> Signal<taffy::Dimension> {
    let val = val.into();
    derive_signal!(taffy::Dimension::Length(val.get()))
}

pub fn pct(val: impl Into<MaybeSignal<f32>>) -> Signal<taffy::Dimension> {
    let val = val.into();
    derive_signal!(taffy::Dimension::Percent(val.get() / 100.0))
}

pub fn length_percentage_pct(val: impl Into<MaybeSignal<f32>>) -> Signal<taffy::LengthPercentage> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentage::Percent(val.get() / 100.0))
}

pub fn length_percentage_chars(
    val: impl Into<MaybeSignal<f32>>,
) -> Signal<taffy::LengthPercentage> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentage::Length(val.get()))
}

pub fn length_percentage_auto_pct(
    val: impl Into<MaybeSignal<f32>>,
) -> Signal<taffy::LengthPercentageAuto> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentageAuto::Percent(val.get() / 100.0))
}

pub fn length_percentage_auto_chars(
    val: impl Into<MaybeSignal<f32>>,
) -> Signal<taffy::LengthPercentageAuto> {
    let val = val.into();
    derive_signal!(taffy::LengthPercentageAuto::Length(val.get()))
}

pub struct Show(MaybeSignal<bool>);

impl UpdateLayout for Show {
    fn update_layout(&self, original_display: taffy::Display, style: &mut taffy::Style) {
        style.display = if self.0.get() {
            original_display
        } else {
            Display::None
        }
    }
}

pub fn show(val: impl Into<MaybeSignal<bool>>) -> (Show,) {
    (Show(val.into()),)
}

pub struct Block(pub(crate) MaybeSignal<ratatui::widgets::Block<'static>>);

pub fn block(block: impl Into<MaybeSignal<ratatui::widgets::Block<'static>>>) -> Block {
    Block(block.into())
}

impl Property for Block {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                nodes.set_block(key, self.0.get());
                nodes.update_layout(key, |s| s.border = taffy::Rect::length(1.));
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct ZIndex(pub(crate) MaybeSignal<i32>);

pub fn z_index(z_index: impl Into<MaybeSignal<i32>>) -> ZIndex {
    ZIndex(z_index.into())
}

impl Property for ZIndex {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                nodes.set_z_index(key, self.0.get());
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct Clear(pub(crate) MaybeSignal<bool>);

pub fn clear(clear: impl Into<MaybeSignal<bool>>) -> Clear {
    Clear(clear.into())
}

impl Property for Clear {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                nodes.set_clear(key, self.0.get());
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

impl<T> Property for T
where
    T: UpdateLayout + 'static,
{
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                let original_display = nodes[key].original_display;
                nodes.update_layout(key, |s| self.update_layout(original_display, s))
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct Focusable(pub(crate) MaybeSignal<bool>);

pub fn focusable(focusable: impl Into<MaybeSignal<bool>>) -> Focusable {
    Focusable(focusable.into())
}

impl Property for Focusable {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                nodes.set_focusable(key, self.0.get());
            });
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
        $crate::layout::width($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::width($crate::layout::chars($val))
    };
}

#[macro_export]
macro_rules! height {
    ($val:tt %) => {
        $crate::layout::height($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::height($crate::layout::chars($val))
    };
}

#[macro_export]
macro_rules! min_width {
    ($val:tt %) => {
        $crate::layout::min_width($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::min_width($crate::layout::chars($val))
    };
}

#[macro_export]
macro_rules! min_height {
    ($val:tt %) => {
        $crate::layout::min_height($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::min_height($crate::layout::chars($val))
    };
}

#[macro_export]
macro_rules! max_width {
    ($val:tt %) => {
        $crate::layout::max_width($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::max_width($crate::layout::chars($val))
    };
}

#[macro_export]
macro_rules! max_height {
    ($val:tt %) => {
        $crate::layout::max_height($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::max_height($crate::layout::chars($val))
    };
}

#[macro_export]
macro_rules! padding_left {
    ($val:tt %) => {
        $crate::layout::padding_left($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding_left($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_right {
    ($val:tt %) => {
        $crate::layout::padding_right($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding_right($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_top {
    ($val:tt %) => {
        $crate::layout::padding_top($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding_top($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_bottom {
    ($val:tt %) => {
        $crate::layout::padding_bottom($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding_bottom($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_x {
    ($val:tt %) => {
        $crate::layout::padding_x($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding_x($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding_y {
    ($val:tt %) => {
        $crate::layout::padding_y($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding_y($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! padding {
    ($val:tt %) => {
        $crate::layout::padding($crate::layout::length_percentage_pct($val))
    };
    ($val:tt) => {
        $crate::layout::padding($crate::layout::length_percentage_chars($val))
    };
}

#[macro_export]
macro_rules! margin {
    ($val:tt %) => {
        $crate::layout::margin($crate::layout::length_percentage_auto_pct($val))
    };
    ($val:tt) => {
        $crate::layout::margin($crate::layout::length_percentage_auto_chars($val))
    };
}

macro_rules! layout_prop {
    ($struct_name:ident, $fn:ident, $inner:ty, $($($props:ident).+),+) => {
        #[derive(Default, Clone)]
        pub struct $struct_name(pub(crate) Option<MaybeSignal<$inner>>);

        impl $struct_name {
            pub fn value(&self) -> Option<MaybeSignal<$inner>> {
                self.0.clone()
            }
        }

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    $(style.$($props).* = inner.get();)+
                }

            }
        }

        pub fn $fn(val: impl Into<MaybeSignal<$inner>>) -> ($struct_name,) {
            ($struct_name(Some(val.into())),)
        }
    };
}

macro_rules! layout_prop_opt {
    ($struct_name:ident, $fn:ident, $inner:ty, $($prop:ident).*) => {
        #[derive(Default, Clone)]
        pub struct $struct_name(pub(crate) Option<MaybeSignal<$inner>>);

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    style.$($prop).* = Some(inner.get());
                }
            }
        }

        pub fn $fn(val: impl Into<MaybeSignal<$inner>>) -> ($struct_name,) {
            ($struct_name(Some(val.into())),)
        }
    };
}

// Generic properties
layout_prop!(Width, width, taffy::Dimension, size.width);
layout_prop!(Height, height, taffy::Dimension, size.height);
layout_prop!(MinWidth, min_width, taffy::Dimension, min_size.width);
layout_prop!(MinHeight, min_height, taffy::Dimension, min_size.height);
layout_prop!(MaxWidth, max_width, taffy::Dimension, max_size.width);
layout_prop!(MaxHeight, max_height, taffy::Dimension, max_size.height);
layout_prop_opt!(AspectRatio, aspect_ratio, f32, aspect_ratio);
layout_prop!(Position, position, taffy::style::Position, position);

layout_prop!(
    MarginLeft,
    margin_left,
    taffy::LengthPercentageAuto,
    margin.left
);
layout_prop!(
    MarginRight,
    margin_right,
    taffy::LengthPercentageAuto,
    margin.right
);
layout_prop!(
    MarginTop,
    margin_top,
    taffy::LengthPercentageAuto,
    margin.top
);
layout_prop!(
    MarginBottom,
    margin_bottom,
    taffy::LengthPercentageAuto,
    margin.bottom
);
layout_prop!(
    MarginX,
    margin_x,
    taffy::LengthPercentageAuto,
    margin.left,
    margin.right
);
layout_prop!(
    MarginY,
    margin_y,
    taffy::LengthPercentageAuto,
    margin.top,
    margin.bottom
);
layout_prop!(
    Margin,
    margin,
    taffy::LengthPercentageAuto,
    margin.top,
    margin.bottom,
    margin.left,
    margin.right
);

layout_prop!(
    PaddingLeft,
    padding_left,
    taffy::LengthPercentage,
    padding.left
);
layout_prop!(
    PaddingRight,
    padding_right,
    taffy::LengthPercentage,
    padding.right
);
layout_prop!(
    PaddingTop,
    padding_top,
    taffy::LengthPercentage,
    padding.top
);
layout_prop!(
    PaddingBottom,
    padding_bottom,
    taffy::LengthPercentage,
    padding.bottom
);
layout_prop!(
    PaddingX,
    padding_x,
    taffy::LengthPercentage,
    padding.left,
    padding.right
);
layout_prop!(
    PaddingY,
    padding_y,
    taffy::LengthPercentage,
    padding.top,
    padding.bottom
);
layout_prop!(
    Padding,
    padding,
    taffy::LengthPercentage,
    padding.top,
    padding.bottom,
    padding.left,
    padding.right
);

layout_prop!(
    BorderLeft,
    border_left,
    taffy::LengthPercentage,
    border.left
);
layout_prop!(
    BorderRight,
    border_right,
    taffy::LengthPercentage,
    border.right
);
layout_prop!(BorderTop, border_top, taffy::LengthPercentage, border.top);
layout_prop!(
    BorderBottom,
    border_bottom,
    taffy::LengthPercentage,
    border.bottom
);
layout_prop!(
    BorderX,
    border_x,
    taffy::LengthPercentage,
    border.left,
    border.right
);
layout_prop!(
    BorderY,
    border_y,
    taffy::LengthPercentage,
    border.top,
    border.bottom
);
layout_prop!(
    Border,
    border,
    taffy::LengthPercentage,
    border.top,
    border.bottom,
    border.left,
    border.right
);

// Flex properties
layout_prop!(Wrap, wrap, taffy::FlexWrap, flex_wrap);
layout_prop_opt!(AlignItems, align_items, taffy::AlignItems, align_items);
layout_prop_opt!(
    AlignContent,
    align_content,
    taffy::AlignContent,
    align_content
);
layout_prop_opt!(
    JustifyContent,
    justify_content,
    taffy::JustifyContent,
    justify_content
);
layout_prop!(Gap, gap, taffy::Size<taffy::LengthPercentage>, gap);
layout_prop!(Grow, grow, f32, flex_grow);
layout_prop!(Shrink, shrink, f32, flex_shrink);
layout_prop_opt!(AlignSelf, align_self, taffy::AlignSelf, align_self);
layout_prop!(Basis, basis, taffy::Dimension, flex_basis);

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
