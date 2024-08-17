use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeSignal, Signal};
use taffy::Display;

use super::{with_nodes_mut, DomNode, Property};
use crate::derive_signal;

trait UpdateLayout {
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

pub struct Hide(MaybeSignal<bool>);

impl UpdateLayout for Hide {
    fn update_layout(&self, original_display: taffy::Display, style: &mut taffy::Style) {
        style.display = if self.0.get() {
            Display::None
        } else {
            original_display
        }
    }
}

pub fn hide(val: impl Into<MaybeSignal<bool>>) -> (Hide,) {
    (Hide(val.into()),)
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
        $crate::layout::min_width($crate::layout::pct($val))
    };
    ($val:tt) => {
        $crate::layout::min_width($crate::layout::chars($val))
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

macro_rules! layout_prop {
    ($struct_name:ident, $fn:ident, $inner:ty, $($prop:ident).*) => {
        #[derive(Default, Clone)]
        pub struct $struct_name(pub(crate) Option<MaybeSignal<$inner>>);

        impl UpdateLayout for $struct_name {
            fn update_layout(&self, _: taffy::Display, style: &mut taffy::Style) {
                if let Some(inner) = self.0 {
                    style.$($prop).* = inner.get();
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
layout_prop!(
    Margin,
    margin,
    taffy::Rect<taffy::LengthPercentageAuto>,
    margin
);
layout_prop!(
    Padding,
    padding,
    taffy::Rect<taffy::LengthPercentage>,
    padding
);
layout_prop!(Border, border, taffy::Rect<taffy::LengthPercentage>, border);
layout_prop!(Position, position, taffy::style::Position, position);

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
