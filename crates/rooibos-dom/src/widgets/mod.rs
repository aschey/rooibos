mod chart;
mod sparkline;

use std::any::type_name;

pub use chart::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget, WidgetRef};
pub use sparkline::*;

use crate::DomWidget;

#[macro_export]
macro_rules! widget_ref {
    ($($x:tt)*) => {
        $crate::widget_ref(move || $($x)*)
    };
}

#[macro_export]
macro_rules! widget {
    ($($x:tt)*) => {
        $crate::widget(move || $($x)*)
    };
}

#[macro_export]
macro_rules! stateful_widget {
    ($x:expr, $y:expr) => {
        $crate::stateful_widget(move || $x, move || $y)
    };
}

pub fn widget_ref<F, W>(props: F) -> DomWidget
where
    F: Fn() -> W + 'static,
    W: WidgetRef + 'static,
{
    DomWidget::new(type_name::<W>(), move || {
        let props = props();
        move |rect: Rect, buf: &mut Buffer| {
            props.render_ref(rect, buf);
        }
    })
}

pub fn widget<F, W>(props: F) -> DomWidget
where
    F: Fn() -> W + 'static,
    W: Widget + Clone + 'static,
{
    DomWidget::new(type_name::<W>(), move || {
        let props = props();
        move |rect: Rect, buf: &mut Buffer| {
            props.clone().render(rect, buf);
        }
    })
}

pub fn stateful_widget<F1, F2, W>(props: F1, state: F2) -> DomWidget
where
    F1: Fn() -> W + 'static,
    F2: Fn() -> W::State + 'static,
    W: StatefulWidget + Clone + 'static,
{
    DomWidget::new(type_name::<W>(), move || {
        let props = props();
        let mut state = state();
        move |rect: Rect, buf: &mut Buffer| {
            props.clone().render(rect, buf, &mut state);
        }
    })
}
