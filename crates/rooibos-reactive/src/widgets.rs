use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget, WidgetRef};
use rooibos_dom::{MeasureNode, RenderNode};

use crate::dom::DomWidget;

#[macro_export]
macro_rules! wgt {
     (props($($properties:expr),+), $state:expr, $($x:tt)*) => {
        $crate::stateful_widget(($($properties),+), move || $($x)*, move || $state)
    };
    (props($($properties:expr),+), $($x:tt)*) => {
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
    (props($($properties:expr),+), $($x:tt)*) => {
        $crate::widget_owned(($($properties),+), move || $($x)*)
    };
    ($($x:tt)*) => {
        $crate::widget_owned((), move || $($x)*)
    };
}

pub(crate) struct RenderWidgetRef<W>
where
    W: WidgetRef + 'static,
{
    pub(crate) widget: W,
}

impl<W> RenderNode for RenderWidgetRef<W>
where
    W: WidgetRef + 'static,
{
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        self.widget.render_ref(rect, frame.buffer_mut())
    }
}

impl<W> MeasureNode for RenderWidgetRef<W>
where
    W: WidgetRef + MeasureNode,
{
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        self.widget
            .measure(known_dimensions, available_space, style)
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        self.widget.estimate_size()
    }
}

pub(crate) struct RenderWidget<W>
where
    W: Widget + 'static,
{
    pub(crate) widget: W,
}

impl<W> RenderNode for RenderWidget<W>
where
    W: Widget + Clone + 'static,
{
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        self.widget.clone().render(rect, frame.buffer_mut())
    }
}

impl<W> MeasureNode for RenderWidget<W>
where
    W: Widget + MeasureNode,
{
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        self.widget
            .measure(known_dimensions, available_space, style)
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        self.widget.estimate_size()
    }
}

pub(crate) struct RenderStatefulWidget<W>
where
    W: StatefulWidget + Clone + 'static,
{
    pub(crate) widget: W,
    pub(crate) state: W::State,
}

impl<W> RenderNode for RenderStatefulWidget<W>
where
    W: StatefulWidget + Clone + 'static,
{
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        self.widget
            .clone()
            .render(rect, frame.buffer_mut(), &mut self.state);
    }
}

impl<W> MeasureNode for RenderStatefulWidget<W>
where
    W: StatefulWidget + Clone + MeasureNode + 'static,
{
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        self.widget
            .measure(known_dimensions, available_space, style)
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        self.widget.estimate_size()
    }
}

pub fn widget<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: WidgetRef + MeasureNode + 'static,
{
    DomWidget::new_with_properties::<W, _>(props, move || RenderWidgetRef {
        widget: widget_props(),
    })
}

pub fn widget_owned<P, F, W>(props: P, widget_props: F) -> DomWidget<P>
where
    F: Fn() -> W + 'static,
    W: Widget + MeasureNode + Clone + 'static,
{
    DomWidget::new_with_properties::<W, _>(props, move || RenderWidget {
        widget: widget_props(),
    })
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
