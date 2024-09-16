use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget, WidgetRef};

use crate::DomWidget;

#[macro_export]
macro_rules! wgt {
    (props($($properties:expr),+), $($x:tt)*) => {
        $crate::widget(($($properties),+), move || $($x)*)
    };
    ($x:expr) => {
        $crate::widget((), move || $x)
    };
    ($x:expr, $y:expr) => {
        $crate::stateful_widget((), move || $x, move || $y)
    };
    (props($($properties:expr),+), $x:expr, $y:expr) => {
        $crate::stateful_widget(($($properties),+), move || $x, move || $y)
    };
}

#[macro_export]
macro_rules! wgt_owned {
    (props($($properties:expr),+), $($x:tt)*) => {
        $crate::widget_owned(($($properties),+), move || $($x)*)
    };
    ($($x:tt)*) => {
        $crate::widget_owned((), move || $($x)*)
    };
}

pub fn widget<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
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

pub fn widget_owned<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
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
