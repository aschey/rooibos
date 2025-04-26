use std::sync::Arc;

use next_tuple::NextTuple;
use reactive_graph::computed::Memo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::signal::{ReadSignal, RwSignal};
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;
pub use rooibos_dom::{BorderType, Borders};
use rooibos_dom::{FocusMode, NodeId};
use taffy::Display;
use wasm_compat::sync::Mutex;

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

macro_rules! signal_wrapper_inner {
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
    };
}

macro_rules! signal_wrapper {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr) => {
        signal_wrapper_inner!($struct_name, $fn, $inner, $default);

        pub fn $fn(val: impl Into<Signal<$inner>>) -> $struct_name {
            $struct_name(Some(val.into()))
        }
    };
}

macro_rules! custom_signal_wrapper {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr, $impl_trait:ident, $f:expr) => {
        signal_wrapper_inner!($struct_name, $fn, $inner, $default);

        pub fn $fn(val: impl $impl_trait) -> $struct_name {
            $struct_name(Some($f(val)))
        }
    };
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
            //let block = border.into_block();

            with_nodes_mut(|nodes| {
                nodes.set_borders(key, border);
                nodes.update_layout(key, |s| s.border = rect);
            });
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

signal_wrapper!(
    Background,
    background,
    ratatui::style::Color,
    ratatui::style::Color::default()
);

impl Property for Background {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            let Some(color) = self.0 else {
                return;
            };
            let color = color.get();
            with_nodes_mut(|nodes| nodes.set_background(key, color));
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
#[derive(Clone)]
pub struct SyncEffect(Arc<Mutex<rooibos_dom::tachyonfx::Effect>>);

#[cfg(feature = "effects")]
impl SyncEffect {
    pub fn new(effect: rooibos_dom::tachyonfx::Effect) -> Self {
        Self(Arc::new(Mutex::new(effect)))
    }
}

#[cfg(feature = "effects")]
impl From<rooibos_dom::tachyonfx::Effect> for SyncEffect {
    fn from(value: rooibos_dom::tachyonfx::Effect) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "effects")]
signal_wrapper!(
    Effect,
    effect,
    SyncEffect,
    SyncEffect(Arc::new(Mutex::new(rooibos_dom::tachyonfx::Effect::new(
        rooibos_dom::tachyonfx::fx::sequence(&[])
    ))))
);

#[cfg(feature = "effects")]
impl Property for Effect {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let key = node.get_key();
        RenderEffect::new(move |_| {
            if let Some(effect) = self.0 {
                with_nodes_mut(|nodes| {
                    nodes.set_effect(key, effect.get().0.lock().clone());
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
                    nodes.set_focus_mode(
                        key,
                        if focusable.get() {
                            FocusMode::Tab
                        } else {
                            FocusMode::None
                        },
                    );
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

macro_rules! update_layout {
    ($struct_name:ident, $($($props:ident).+),+) => {
        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    $(style.$($props).* = inner.get().into();)+
                }

            }
        }
    }
}

macro_rules! update_layout_opt {
    ($struct_name:ident, $($($props:ident).+),+) => {
        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    $(style.$($props).* = Some(inner.get().into());)+
                }
            }
        }
    };
}

macro_rules! layout_prop {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr, $($props:tt)+) => {
        signal_wrapper!($struct_name, $fn, $inner, $default);
        update_layout!($struct_name, $($props)+);
    };
}

macro_rules! dimension_layout_prop {
    ($struct_name:ident, $fn:ident, $default:expr, $($props:tt)+) => {
        custom_signal_wrapper!($struct_name, $fn, $crate::dom::layout::Dimension, $default, IntoDimensionSignal, IntoDimensionSignal::into_dimension_signal);
        update_layout!($struct_name, $($props)+);
    };
}

macro_rules! custom_layout_prop_opt {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr, $impl_trait:ident, $f:expr,$($props:tt)+) => {
        custom_signal_wrapper!($struct_name, $fn, $inner, $default, $impl_trait, $f);
        update_layout_opt!($struct_name, $($props)+);
    };
}

macro_rules! impl_signal_into {
    ($trait_name:ident, $trait_fn:ident, $impl_ty:ty) => {
        impl $trait_name for Signal<$impl_ty> {
            fn $trait_fn(self) -> Signal<$impl_ty> {
                self
            }
        }

        impl $trait_name for ReadSignal<$impl_ty> {
            fn $trait_fn(self) -> Signal<$impl_ty> {
                self.into()
            }
        }

        impl $trait_name for RwSignal<$impl_ty> {
            fn $trait_fn(self) -> Signal<$impl_ty> {
                self.into()
            }
        }

        impl $trait_name for Memo<$impl_ty> {
            fn $trait_fn(self) -> Signal<$impl_ty> {
                self.into()
            }
        }
    };
}

macro_rules! impl_signal_derives {
    ($trait_name:ident, $trait_fn:ident, $ret_ty:ty, $impl_ty:ty) => {
        impl $trait_name for Signal<$impl_ty> {
            fn $trait_fn(self) -> Signal<$ret_ty> {
                derive_signal!(self.get().into())
            }
        }

        impl $trait_name for ReadSignal<$impl_ty> {
            fn $trait_fn(self) -> Signal<$ret_ty> {
                derive_signal!(self.get().into())
            }
        }

        impl $trait_name for RwSignal<$impl_ty> {
            fn $trait_fn(self) -> Signal<$ret_ty> {
                derive_signal!(self.get().into())
            }
        }

        impl $trait_name for Memo<$impl_ty> {
            fn $trait_fn(self) -> Signal<$ret_ty> {
                derive_signal!(self.get().into())
            }
        }
    };
}

macro_rules! impl_trait_into {
    ($trait_name:ident, $trait_fn:ident, $ret_ty:ty, $impl_ty:ty) => {
        impl $trait_name for $impl_ty {
            fn $trait_fn(self) -> Signal<$ret_ty> {
                let val: $ret_ty = self.into();
                val.into()
            }
        }

        impl_signal_derives!($trait_name, $trait_fn, $ret_ty, $impl_ty);
    };
}

macro_rules! layout_prop_opt {
    ($struct_name:ident, $fn:ident, $inner:ty, $default:expr, $($prop:tt)+) => {
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

        update_layout_opt!($struct_name, $($prop)+);

        impl_next_tuple!($struct_name);

        pub fn $fn(val: impl Into<Signal<$inner>>) -> $struct_name {
            $struct_name(Some(val.into()))
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dimension {
    Chars(u32),
    Percent(u32),
    Auto,
}

impl From<taffy::CompactLength> for Dimension {
    fn from(value: taffy::CompactLength) -> Self {
        if value.is_auto() {
            Dimension::Auto
        } else if let Some(v) = value.resolved_percentage_size(1.0, |_, v| v) {
            Dimension::Percent((v * 100.0) as u32)
        } else {
            Dimension::Chars(value.value() as u32)
        }
    }
}

impl From<taffy::Dimension> for Dimension {
    fn from(value: taffy::Dimension) -> Self {
        value.into_raw().into()
    }
}

impl From<Dimension> for taffy::Dimension {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Chars(val) => taffy::Dimension::length(val as f32),
            Dimension::Percent(val) => taffy::Dimension::percent(val as f32 / 100.0),
        }
    }
}

impl From<taffy::LengthPercentageAuto> for Dimension {
    fn from(value: taffy::LengthPercentageAuto) -> Self {
        value.into_raw().into()
    }
}

impl From<Dimension> for taffy::LengthPercentageAuto {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::LengthPercentageAuto::auto(),
            Dimension::Chars(val) => taffy::LengthPercentageAuto::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentageAuto::percent(val as f32 / 100.0),
        }
    }
}

impl From<taffy::LengthPercentage> for Dimension {
    fn from(value: taffy::LengthPercentage) -> Self {
        value.into_raw().into()
    }
}

impl From<Dimension> for taffy::LengthPercentage {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::LengthPercentage::length(0.0),
            Dimension::Chars(val) => taffy::LengthPercentage::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentage::percent(val as f32 / 100.0),
        }
    }
}

impl From<u32> for Dimension {
    fn from(value: u32) -> Self {
        Dimension::Chars(value)
    }
}

impl From<&str> for Dimension {
    fn from(value: &str) -> Self {
        if value.ends_with("%") {
            Dimension::Percent(value.strip_suffix("%").unwrap().parse().unwrap())
        } else if value.eq_ignore_ascii_case("auto") {
            Dimension::Auto
        } else {
            Dimension::Chars(value.parse().unwrap())
        }
    }
}

impl From<String> for Dimension {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

pub const fn pct(val: u32) -> Dimension {
    Dimension::Percent(val)
}

pub const fn full() -> Dimension {
    Dimension::Percent(100)
}

pub const fn half() -> Dimension {
    Dimension::Percent(50)
}

pub fn chars(val: impl Into<Signal<u32>>) -> Signal<Dimension> {
    let val = val.into();
    derive_signal!(Dimension::Chars(val.get()))
}

pub trait IntoDimensionSignal {
    fn into_dimension_signal(self) -> Signal<Dimension>;
}

impl_signal_into!(IntoDimensionSignal, into_dimension_signal, Dimension);

impl_signal_derives!(IntoDimensionSignal, into_dimension_signal, Dimension, u32);

impl_signal_derives!(
    IntoDimensionSignal,
    into_dimension_signal,
    Dimension,
    &'static str
);

impl_signal_derives!(
    IntoDimensionSignal,
    into_dimension_signal,
    Dimension,
    String
);

impl IntoDimensionSignal for u32 {
    fn into_dimension_signal(self) -> Signal<Dimension> {
        let dim: Dimension = self.into();
        dim.into()
    }
}

impl IntoDimensionSignal for &str {
    fn into_dimension_signal(self) -> Signal<Dimension> {
        let dim: Dimension = self.into();
        dim.into()
    }
}

impl IntoDimensionSignal for String {
    fn into_dimension_signal(self) -> Signal<Dimension> {
        let dim: Dimension = self.into();
        dim.into()
    }
}

impl IntoDimensionSignal for Dimension {
    fn into_dimension_signal(self) -> Signal<Dimension> {
        self.into()
    }
}

pub fn val(val: impl IntoDimensionSignal) -> Signal<Dimension> {
    val.into_dimension_signal()
}

pub const fn auto() -> Dimension {
    Dimension::Auto
}

pub const fn scroll() -> taffy::Overflow {
    taffy::Overflow::Scroll
}

pub const fn clip() -> taffy::Overflow {
    taffy::Overflow::Clip
}

pub const fn visible() -> taffy::Overflow {
    taffy::Overflow::Visible
}

pub const fn hidden() -> taffy::Overflow {
    taffy::Overflow::Hidden
}

pub const fn absolute() -> taffy::Position {
    taffy::Position::Absolute
}

pub const fn wrap() -> taffy::FlexWrap {
    taffy::FlexWrap::Wrap
}

pub const fn no_wrap() -> taffy::FlexWrap {
    taffy::FlexWrap::NoWrap
}

pub const fn wrap_reverse() -> taffy::FlexWrap {
    taffy::FlexWrap::WrapReverse
}

pub trait IntoAlignItemsSignal {
    fn into_align_items_signal(self) -> Signal<taffy::AlignItems>;
}

pub trait IntoJustifyContentSignal {
    fn into_justify_content_signal(self) -> Signal<taffy::JustifyContent>;
}

pub trait IntoJustifyItemsSignal {
    fn into_justify_items_signal(self) -> Signal<taffy::JustifyItems>;
}

pub trait IntoJustifySelfSignal {
    fn into_justify_self_signal(self) -> Signal<taffy::JustifySelf>;
}

pub trait IntoAlignSelfSignal {
    fn into_align_self_signal(self) -> Signal<taffy::AlignSelf>;
}

pub trait IntoAlignContentSignal {
    fn into_align_content_signal(self) -> Signal<taffy::AlignContent>;
}

impl_signal_into!(
    IntoAlignItemsSignal,
    into_align_items_signal,
    taffy::AlignItems
);
impl_signal_into!(
    IntoAlignContentSignal,
    into_align_content_signal,
    taffy::AlignContent
);
impl_signal_into!(
    IntoJustifyItemsSignal,
    into_justify_items_signal,
    taffy::JustifyItems
);
impl_signal_into!(
    IntoJustifyContentSignal,
    into_justify_content_signal,
    taffy::JustifyContent
);
impl_signal_into!(
    IntoAlignSelfSignal,
    into_align_self_signal,
    taffy::AlignSelf
);
impl_signal_into!(
    IntoJustifySelfSignal,
    into_justify_self_signal,
    taffy::JustifySelf
);

impl_trait_into!(
    IntoAlignItemsSignal,
    into_align_items_signal,
    taffy::AlignItems,
    Start
);
impl_trait_into!(
    IntoAlignContentSignal,
    into_align_content_signal,
    taffy::AlignContent,
    Start
);
impl_trait_into!(
    IntoJustifyItemsSignal,
    into_justify_items_signal,
    taffy::JustifyItems,
    Start
);
impl_trait_into!(
    IntoJustifyContentSignal,
    into_justify_content_signal,
    taffy::JustifyContent,
    Start
);
impl_trait_into!(
    IntoAlignSelfSignal,
    into_align_self_signal,
    taffy::AlignSelf,
    Start
);
impl_trait_into!(
    IntoJustifySelfSignal,
    into_justify_self_signal,
    taffy::JustifySelf,
    Start
);

impl_trait_into!(
    IntoAlignItemsSignal,
    into_align_items_signal,
    taffy::AlignItems,
    End
);
impl_trait_into!(
    IntoAlignContentSignal,
    into_align_content_signal,
    taffy::AlignContent,
    End
);
impl_trait_into!(
    IntoJustifyItemsSignal,
    into_justify_items_signal,
    taffy::JustifyItems,
    End
);
impl_trait_into!(
    IntoJustifyContentSignal,
    into_justify_content_signal,
    taffy::JustifyContent,
    End
);
impl_trait_into!(
    IntoAlignSelfSignal,
    into_align_self_signal,
    taffy::AlignSelf,
    End
);
impl_trait_into!(
    IntoJustifySelfSignal,
    into_justify_self_signal,
    taffy::JustifySelf,
    End
);

impl_trait_into!(
    IntoAlignItemsSignal,
    into_align_items_signal,
    taffy::AlignItems,
    Center
);
impl_trait_into!(
    IntoAlignContentSignal,
    into_align_content_signal,
    taffy::AlignContent,
    Center
);
impl_trait_into!(
    IntoJustifyItemsSignal,
    into_justify_items_signal,
    taffy::JustifyItems,
    Center
);
impl_trait_into!(
    IntoJustifyContentSignal,
    into_justify_content_signal,
    taffy::JustifyContent,
    Center
);
impl_trait_into!(
    IntoAlignSelfSignal,
    into_align_self_signal,
    taffy::AlignSelf,
    Center
);
impl_trait_into!(
    IntoJustifySelfSignal,
    into_justify_self_signal,
    taffy::JustifySelf,
    Center
);

impl_trait_into!(
    IntoAlignItemsSignal,
    into_align_items_signal,
    taffy::AlignItems,
    FlexStart
);
impl_trait_into!(
    IntoAlignContentSignal,
    into_align_content_signal,
    taffy::AlignContent,
    FlexStart
);
impl_trait_into!(
    IntoJustifyItemsSignal,
    into_justify_items_signal,
    taffy::JustifyItems,
    FlexStart
);
impl_trait_into!(
    IntoJustifyContentSignal,
    into_justify_content_signal,
    taffy::JustifyContent,
    FlexStart
);
impl_trait_into!(
    IntoAlignSelfSignal,
    into_align_self_signal,
    taffy::AlignSelf,
    FlexStart
);
impl_trait_into!(
    IntoJustifySelfSignal,
    into_justify_self_signal,
    taffy::JustifySelf,
    FlexStart
);

impl_trait_into!(
    IntoAlignItemsSignal,
    into_align_items_signal,
    taffy::AlignItems,
    FlexEnd
);
impl_trait_into!(
    IntoAlignContentSignal,
    into_align_content_signal,
    taffy::AlignContent,
    FlexEnd
);
impl_trait_into!(
    IntoJustifyItemsSignal,
    into_justify_items_signal,
    taffy::JustifyItems,
    FlexEnd
);
impl_trait_into!(
    IntoJustifyContentSignal,
    into_justify_content_signal,
    taffy::JustifyContent,
    FlexEnd
);
impl_trait_into!(
    IntoAlignSelfSignal,
    into_align_self_signal,
    taffy::AlignSelf,
    FlexEnd
);
impl_trait_into!(
    IntoJustifySelfSignal,
    into_justify_self_signal,
    taffy::JustifySelf,
    FlexEnd
);

#[derive(Clone, Copy)]
pub struct Start;

pub const fn start() -> Start {
    Start
}

impl From<Start> for taffy::AlignItems {
    fn from(_value: Start) -> Self {
        taffy::AlignItems::Start
    }
}

impl From<Start> for taffy::AlignContent {
    fn from(_value: Start) -> Self {
        taffy::AlignContent::Start
    }
}

#[derive(Clone, Copy)]
pub struct End;

pub const fn end() -> End {
    End
}

impl From<End> for taffy::AlignItems {
    fn from(_value: End) -> Self {
        taffy::AlignItems::End
    }
}

impl From<End> for Signal<taffy::AlignItems> {
    fn from(_value: End) -> Self {
        taffy::AlignItems::End.into()
    }
}

impl From<End> for taffy::AlignContent {
    fn from(_value: End) -> Self {
        taffy::AlignContent::End
    }
}

impl From<End> for Signal<taffy::AlignContent> {
    fn from(_value: End) -> Self {
        taffy::AlignContent::End.into()
    }
}

#[derive(Clone, Copy)]
pub struct FlexStart;

pub const fn flex_start() -> FlexStart {
    FlexStart
}

impl From<FlexStart> for taffy::AlignItems {
    fn from(_value: FlexStart) -> Self {
        taffy::AlignItems::FlexStart
    }
}

impl From<FlexStart> for taffy::AlignContent {
    fn from(_value: FlexStart) -> Self {
        taffy::AlignContent::FlexStart
    }
}

#[derive(Clone, Copy)]
pub struct FlexEnd;

pub const fn flex_end() -> FlexEnd {
    FlexEnd
}

impl From<FlexEnd> for taffy::AlignItems {
    fn from(_value: FlexEnd) -> Self {
        taffy::AlignItems::FlexEnd
    }
}

impl From<FlexEnd> for taffy::AlignContent {
    fn from(_value: FlexEnd) -> Self {
        taffy::AlignContent::FlexEnd
    }
}

#[derive(Clone, Copy)]
pub struct Center;

pub const fn center() -> Center {
    Center
}

impl From<Center> for taffy::AlignItems {
    fn from(_value: Center) -> Self {
        taffy::AlignItems::Center
    }
}

impl From<Center> for taffy::AlignContent {
    fn from(_value: Center) -> Self {
        taffy::AlignContent::Center
    }
}

pub struct Baseline;

pub const fn baseline() -> Baseline {
    Baseline
}

impl From<Baseline> for taffy::AlignItems {
    fn from(_value: Baseline) -> Self {
        taffy::AlignItems::Baseline
    }
}

pub struct Stretch;

pub const fn stretch() -> Stretch {
    Stretch
}

impl From<Stretch> for taffy::AlignItems {
    fn from(_value: Stretch) -> Self {
        taffy::AlignItems::Stretch
    }
}

impl From<Stretch> for taffy::AlignContent {
    fn from(_value: Stretch) -> Self {
        taffy::AlignContent::Stretch
    }
}

pub struct SpaceBetween;

pub const fn space_between() -> SpaceBetween {
    SpaceBetween
}

impl From<SpaceBetween> for taffy::AlignContent {
    fn from(_value: SpaceBetween) -> Self {
        taffy::AlignContent::SpaceBetween
    }
}

pub struct SpaceEvenly;

pub const fn space_evenly() -> SpaceEvenly {
    SpaceEvenly
}

impl From<SpaceEvenly> for taffy::AlignContent {
    fn from(_value: SpaceEvenly) -> Self {
        taffy::AlignContent::SpaceEvenly
    }
}

pub struct SpaceAround;

pub const fn space_around() -> SpaceAround {
    SpaceAround
}

impl From<SpaceAround> for taffy::AlignContent {
    fn from(_value: SpaceAround) -> Self {
        taffy::AlignContent::SpaceAround
    }
}

// Generic properties
dimension_layout_prop!(Width, width, Dimension::Auto, size.width);
dimension_layout_prop!(Height, height, Dimension::Auto, size.height);
dimension_layout_prop!(MinWidth, min_width, Dimension::Auto, min_size.width);
dimension_layout_prop!(MinHeight, min_height, Dimension::Auto, min_size.height);
dimension_layout_prop!(MaxWidth, max_width, Dimension::Auto, max_size.width);
dimension_layout_prop!(MaxHeight, max_height, Dimension::Auto, max_size.height);
layout_prop_opt!(AspectRatio, aspect_ratio, f32, 0.0, aspect_ratio);
layout_prop!(
    Position,
    position,
    taffy::style::Position,
    taffy::style::Position::default(),
    position
);

dimension_layout_prop!(MarginLeft, margin_left, Dimension::Auto, margin.left);
dimension_layout_prop!(MarginRight, margin_right, Dimension::Auto, margin.right);
dimension_layout_prop!(MarginTop, margin_top, Dimension::Auto, margin.top);
dimension_layout_prop!(MarginBottom, margin_bottom, Dimension::Auto, margin.bottom);
dimension_layout_prop!(
    MarginX,
    margin_x,
    Dimension::Auto,
    margin.left,
    margin.right
);
dimension_layout_prop!(
    MarginY,
    margin_y,
    Dimension::Auto,
    margin.top,
    margin.bottom
);
dimension_layout_prop!(
    Margin,
    margin,
    Dimension::Auto,
    margin.top,
    margin.bottom,
    margin.left,
    margin.right
);

dimension_layout_prop!(PaddingLeft, padding_left, Dimension::Chars(0), padding.left);
dimension_layout_prop!(
    PaddingRight,
    padding_right,
    Dimension::Chars(0),
    padding.right
);
dimension_layout_prop!(PaddingTop, padding_top, Dimension::Chars(0), padding.top);
dimension_layout_prop!(
    PaddingBottom,
    padding_bottom,
    Dimension::Chars(0),
    padding.bottom
);
dimension_layout_prop!(
    PaddingX,
    padding_x,
    Dimension::Chars(0),
    padding.left,
    padding.right
);
dimension_layout_prop!(
    PaddingY,
    padding_y,
    Dimension::Chars(0),
    padding.top,
    padding.bottom
);
dimension_layout_prop!(
    Padding,
    padding,
    Dimension::Chars(0),
    padding.top,
    padding.bottom,
    padding.left,
    padding.right
);
dimension_layout_prop!(ColumnGap, column_gap, Dimension::Chars(0), gap.width);
dimension_layout_prop!(RowGap, row_gap, Dimension::Chars(0), gap.height);
dimension_layout_prop!(Gap, gap, Dimension::Chars(0), gap.width, gap.height);

dimension_layout_prop!(FlexBasis, flex_basis, Dimension::Auto, flex_basis);

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

// align items - aligns items along cross axis
// align content - distribution of space around content
// justify items -
// justify content - align items along main axis
// align self - align self along cross axis
// justify self - ignored in flexbox

pub fn align_cross(val: impl IntoAlignItemsSignal) -> AlignItems {
    align_items(val)
}

pub fn align_main(val: impl IntoJustifyContentSignal) -> JustifyContent {
    justify_content(val)
}

pub fn align_container(val: impl IntoAlignContentSignal) -> AlignContent {
    align_content(val)
}

// Flex properties
layout_prop!(
    FlexWrap,
    flex_wrap,
    taffy::FlexWrap,
    taffy::FlexWrap::default(),
    flex_wrap
);
custom_layout_prop_opt!(
    AlignItems,
    align_items,
    taffy::AlignItems,
    taffy::AlignItems::Start,
    IntoAlignItemsSignal,
    IntoAlignItemsSignal::into_align_items_signal,
    align_items
);
custom_layout_prop_opt!(
    AlignContent,
    align_content,
    taffy::AlignContent,
    taffy::AlignContent::Start,
    IntoAlignContentSignal,
    IntoAlignContentSignal::into_align_content_signal,
    align_content
);
custom_layout_prop_opt!(
    JustifyContent,
    justify_content,
    taffy::JustifyContent,
    taffy::JustifyContent::Start,
    IntoJustifyContentSignal,
    IntoJustifyContentSignal::into_justify_content_signal,
    justify_content
);
custom_layout_prop_opt!(
    JustifyItems,
    justify_items,
    taffy::JustifyItems,
    taffy::JustifyItems::Start,
    IntoJustifyItemsSignal,
    IntoJustifyItemsSignal::into_justify_items_signal,
    justify_items
);
layout_prop!(FlexGrow, flex_grow, f32, 0.0, flex_grow);
layout_prop!(FlexShrink, flex_shrink, f32, 0.0, flex_shrink);
custom_layout_prop_opt!(
    AlignSelf,
    align_self,
    taffy::AlignSelf,
    taffy::AlignSelf::Start,
    IntoAlignSelfSignal,
    IntoAlignSelfSignal::into_align_self_signal,
    align_self
);
custom_layout_prop_opt!(
    JustifySelf,
    justify_self,
    taffy::JustifySelf,
    taffy::JustifySelf::Start,
    IntoJustifySelfSignal,
    IntoJustifySelfSignal::into_justify_self_signal,
    justify_self
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
