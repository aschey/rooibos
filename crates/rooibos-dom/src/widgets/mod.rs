mod button;
mod chart;
mod sparkline;

use std::any::{Any, TypeId};

pub use button::*;
pub use chart::*;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, StatefulWidget, Tabs, Widget, WidgetRef};
pub use sparkline::*;
use taffy::Size;

use crate::{MeasureNode, RenderNode};

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

impl MeasureNode for Tabs<'_> {
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl MeasureNode for List<'_> {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

pub struct RenderWidgetRef<W>(pub W)
where
    W: WidgetRef + 'static;

impl<W> RenderNode for RenderWidgetRef<W>
where
    W: WidgetRef + 'static,
{
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        self.0.render_ref(rect, frame.buffer_mut())
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
        self.0.measure(known_dimensions, available_space, style)
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        self.0.estimate_size()
    }
}

pub struct RenderWidget<W>(pub W)
where
    W: Widget + 'static;

impl<W> RenderNode for RenderWidget<W>
where
    W: Widget + Clone + 'static,
{
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        self.0.clone().render(rect, frame.buffer_mut())
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
        self.0.measure(known_dimensions, available_space, style)
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        self.0.estimate_size()
    }
}

pub struct RenderStatefulWidget<W>
where
    W: StatefulWidget + Clone + 'static,
{
    pub widget: W,
    pub state: W::State,
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
