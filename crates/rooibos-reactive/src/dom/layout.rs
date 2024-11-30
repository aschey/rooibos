use ratatui::layout;
use ratatui::style::{Style, Styled};
use ratatui::symbols::{block, border};
use ratatui::text::{Line, Text};
use ratatui::widgets::block::{Title, title};
use reactive_graph::computed::Memo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::signal::{ArcRwSignal, RwSignal};
use reactive_graph::traits::{Get, Update};
use reactive_graph::wrappers::read::{ArcSignal, MaybeSignal, Signal};
use taffy::{Display, LengthPercentage};

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

impl Property for Show {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                let original_display = *nodes.original_display(key);
                let enabled = self.0.get();
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

pub fn show(val: impl Into<MaybeSignal<bool>>) -> (Show,) {
    (Show(val.into()),)
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BorderType {
    #[default]
    Solid,
    Round,
    Double,
    Thick,
    Outer,
    Inner,
    Wide,
    Tall,
    ThickWide,
    ThickTall,
    ThickFull,
    Empty,
}

impl From<BorderType> for border::Set {
    fn from(value: BorderType) -> Self {
        match value {
            BorderType::Solid => border::PLAIN,
            BorderType::Round => border::ROUNDED,
            BorderType::Double => border::DOUBLE,
            BorderType::Thick => border::THICK,
            BorderType::Outer => border::QUADRANT_OUTSIDE,
            BorderType::Inner => border::QUADRANT_INSIDE,
            BorderType::Wide => border::ONE_EIGHTH_WIDE,
            BorderType::Tall => border::ONE_EIGHTH_TALL,
            BorderType::ThickWide => border::PROPORTIONAL_WIDE,
            BorderType::ThickTall => border::PROPORTIONAL_TALL,
            BorderType::ThickFull => border::FULL,
            BorderType::Empty => border::EMPTY,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Borders {
    borders: ratatui::widgets::Borders,
    border_type: BorderType,
    titles: Vec<Title<'static>>,
    style: Style,
}

impl Styled for Borders {
    type Item = Self;
    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl Borders {
    pub fn all() -> Self {
        Self {
            borders: ratatui::widgets::Borders::ALL,
            ..Default::default()
        }
    }

    pub fn top() -> Self {
        Self {
            borders: ratatui::widgets::Borders::TOP,
            ..Default::default()
        }
    }

    pub fn bottom() -> Self {
        Self {
            borders: ratatui::widgets::Borders::BOTTOM,
            ..Default::default()
        }
    }

    pub fn left() -> Self {
        Self {
            borders: ratatui::widgets::Borders::LEFT,
            ..Default::default()
        }
    }

    pub fn right() -> Self {
        Self {
            borders: ratatui::widgets::Borders::RIGHT,
            ..Default::default()
        }
    }

    pub fn x() -> Self {
        Self {
            borders: ratatui::widgets::Borders::LEFT | ratatui::widgets::Borders::RIGHT,
            ..Default::default()
        }
    }

    pub fn y() -> Self {
        Self {
            borders: ratatui::widgets::Borders::TOP | ratatui::widgets::Borders::BOTTOM,
            ..Default::default()
        }
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    pub fn solid(self) -> Self {
        self.border_type(BorderType::Solid)
    }

    pub fn round(self) -> Self {
        self.border_type(BorderType::Round)
    }

    pub fn double(self) -> Self {
        self.border_type(BorderType::Double)
    }

    pub fn thick(self) -> Self {
        self.border_type(BorderType::Thick)
    }

    pub fn outer(self) -> Self {
        self.border_type(BorderType::Outer)
    }

    pub fn inner(self) -> Self {
        self.border_type(BorderType::Inner)
    }

    pub fn wide(self) -> Self {
        self.border_type(BorderType::Wide)
    }

    pub fn tall(self) -> Self {
        self.border_type(BorderType::Tall)
    }

    pub fn thick_wide(self) -> Self {
        self.border_type(BorderType::ThickWide)
    }

    pub fn thick_tall(self) -> Self {
        self.border_type(BorderType::ThickTall)
    }

    pub fn thick_full(self) -> Self {
        self.border_type(BorderType::ThickFull)
    }

    pub fn empty(self) -> Self {
        self.border_type(BorderType::Empty)
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Title<'static>>,
    {
        self.titles.push(title.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

pub struct BorderProp(pub(crate) MaybeSignal<Borders>);

pub fn borders<S>(borders: S) -> (BorderProp,)
where
    S: Into<MaybeSignal<Borders>>,
{
    (BorderProp(borders.into()),)
}

impl Property for BorderProp {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            let border = self.0.get();
            let mut block = ratatui::widgets::Block::new()
                .borders(border.borders)
                .border_set(border.border_type.into())
                .border_style(border.style);

            for title in border.titles {
                block = block.title(title);
            }

            with_nodes_mut(|nodes| {
                nodes.set_block(key, block);
                let set = LengthPercentage::Length(1.0);
                let mut rect = taffy::Rect::zero();
                let borders = border.borders;
                if borders.intersects(ratatui::widgets::Borders::TOP) {
                    rect.top = set;
                }
                if borders.intersects(ratatui::widgets::Borders::BOTTOM) {
                    rect.bottom = set;
                }
                if borders.intersects(ratatui::widgets::Borders::LEFT) {
                    rect.left = set;
                }
                if borders.intersects(ratatui::widgets::Borders::RIGHT) {
                    rect.right = set;
                }
                nodes.update_layout(key, |s| s.border = rect);
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
        let key = node.get_key();
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
        let key = node.get_key();
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

pub struct Focusable(pub(crate) MaybeSignal<bool>);

pub fn focusable(focusable: impl Into<MaybeSignal<bool>>) -> Focusable {
    Focusable(focusable.into())
}

impl Property for Focusable {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
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

pub struct Enabled(pub(crate) MaybeSignal<bool>);

pub fn enabled(enabled: impl Into<MaybeSignal<bool>>) -> Enabled {
    Enabled(enabled.into())
}

impl Property for Enabled {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        let enabled = Memo::new(move |_| self.0.get());
        RenderEffect::new(move |_| {
            with_nodes_mut(|nodes| {
                nodes.set_enabled(key, enabled.get());
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
