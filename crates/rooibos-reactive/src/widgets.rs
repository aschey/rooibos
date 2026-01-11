use ratatui::widgets::{StatefulWidget, Widget};
use rooibos_dom::MeasureNode;
use rooibos_dom::widgets::{
    RenderStatefulWidget, RenderWidget, RenderWidgetMark, RenderWidgetRef, WidgetRole,
};

use crate::dom::DomWidget;

#[macro_export]
macro_rules! wgt {
    (style($($properties:expr),+ $(,)?), $state:expr, $($x:tt)*) => {
        $crate::stateful_widget(($($properties),+), move || $($x)*, move || $state)
    };
    (style($($properties:expr),+ $(,)?), $($x:tt)*) => {
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
    (style($($properties:expr),+ $(,)?), $($x:tt)*) => {
        $crate::widget_owned(($($properties),+), move || $($x)*)
    };
    ($($x:tt)*) => {
        $crate::widget_owned((), move || $($x)*)
    };
}

pub fn widget<P, F, W, M>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: RenderWidgetMark<M> + WidgetRole + MeasureNode + 'static,
    M: 'static,
{
    DomWidget::new_with_properties(props, move || RenderWidgetRef::new(widget_props()))
}

pub fn widget_owned<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: Widget + MeasureNode + WidgetRole + Clone + 'static,
{
    DomWidget::new_with_properties(props, move || RenderWidget(widget_props()))
}

pub fn stateful_widget<P, F1, F2, W>(props: P, widget_props: F1, state: F2) -> DomWidget<P>
where
    F1: Fn() -> W + 'static,
    F2: Fn() -> W::State + 'static,
    W: StatefulWidget + MeasureNode + WidgetRole + Clone + 'static,
    <W as StatefulWidget>::State: Sized,
{
    DomWidget::new_with_properties(props, move || RenderStatefulWidget {
        widget: widget_props(),
        state: state(),
    })
}
