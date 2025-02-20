use ratatui::widgets::{StatefulWidget, Widget, WidgetRef};
use rooibos_dom::MeasureNode;
use rooibos_dom::widgets::{RenderStatefulWidget, RenderWidget, RenderWidgetRef};

use crate::dom::DomWidget;

#[macro_export]
macro_rules! wgt {
    (props($($properties:expr),+ $(,)?), $state:expr, $($x:tt)*) => {
        $crate::stateful_widget(($($properties),+), move || $($x)*, move || $state)
    };
    (props($($properties:expr),+ $(,)?), $($x:tt)*) => {
        $crate::widget(($($properties),+), move || $($x)*)
    };
    ($x:expr) => {
        $crate::widget((), move || $x)
    };
    ($state:expr, $($x:tt)*) => {
        $crate::stateful_widget((), move || $($x)*, move || $state)
    };
}

#[macro_export]
macro_rules! wgt_owned {
    (props($($properties:expr),+ $(,)?), $($x:tt)*) => {
        $crate::widget_owned(($($properties),+), move || $($x)*)
    };
    ($($x:tt)*) => {
        $crate::widget_owned((), move || $($x)*)
    };
}

pub fn widget<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: WidgetRef + MeasureNode + 'static,
{
    DomWidget::new_with_properties::<W, _>(props, move || RenderWidgetRef(widget_props()))
}

pub fn widget_owned<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: Widget + MeasureNode + Clone + 'static,
{
    DomWidget::new_with_properties::<W, _>(props, move || RenderWidget(widget_props()))
}

pub fn stateful_widget<P, F1, F2, W>(props: P, widget_props: F1, state: F2) -> DomWidget<P>
where
    F1: Fn() -> W + 'static,
    F2: Fn() -> W::State + 'static,
    W: StatefulWidget + MeasureNode + Clone + 'static,
{
    DomWidget::new_with_properties::<W, _>(props, move || RenderStatefulWidget {
        widget: widget_props(),
        state: state(),
    })
}
