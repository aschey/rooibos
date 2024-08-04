mod button;
mod chart;
mod sparkline;

use std::any::{Any, TypeId};

pub use button::*;
pub use chart::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget, WidgetRef};
pub use sparkline::*;

use crate::DomWidget;

#[macro_export]
macro_rules! widget_ref {
    (props($($properties:expr),+), $($x:tt)*) => {
        $crate::widget_ref(($($properties),+), move || $($x)*)
    };
    ($($x:tt)*) => {
        $crate::widget_ref((), move || $($x)*)
    };
}

#[macro_export]
macro_rules! widget {
    (props($($properties:expr),+), $($x:tt)*) => {
        $crate::widget(($($properties),+), move || $($x)*)
    };
    ($($x:tt)*) => {
        $crate::widget((), move || $($x)*)
    };
}

#[macro_export]
macro_rules! stateful_widget {
    ($x:expr, $y:expr) => {
        $crate::stateful_widget((), move || $x, move || $y)
    };
    (props($($properties:expr),+), $x:expr, $y:expr) => {
        $crate::stateful_widget(($($properties),+), move || $x, move || $y)
    };
}

pub fn widget_ref<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: WidgetRef + 'static,
{
    DomWidget::new_with_properties::<W, _, _>(props, move || {
        let props = widget_props();
        move |rect: Rect, buf: &mut Buffer| {
            props.render_ref(rect, buf);
        }
    })
}

pub fn widget<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: Widget + Clone + 'static,
{
    DomWidget::new_with_properties::<W, _, _>(props, move || {
        let props = widget_props();
        move |rect: Rect, buf: &mut Buffer| {
            props.clone().render(rect, buf);
        }
    })
}

pub fn stateful_widget<P, F1, F2, W>(props: P, widget_props: F1, state: F2) -> DomWidget<P>
where
    F1: Fn() -> W + 'static,
    F2: Fn() -> W::State + 'static,
    W: StatefulWidget + Clone + 'static,
{
    DomWidget::new_with_properties::<W, _, _>(props, move || {
        let props = widget_props();
        let mut state = state();
        move |rect: Rect, buf: &mut Buffer| {
            props.clone().render(rect, buf, &mut state);
        }
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Button,
    TextInput,
}

pub trait WidgetRole {
    fn widget_role() -> Option<Role>;
}

impl<T> WidgetRole for T
where
    T: Any,
{
    fn widget_role() -> Option<Role> {
        let current_type = TypeId::of::<Self>();

        if current_type == TypeId::of::<Button>() {
            return Some(Role::Button);
        }
        #[cfg(feature = "input")]
        if current_type == TypeId::of::<tui_textarea::TextArea>() {
            return Some(Role::TextInput);
        }
        None
    }
}
