mod button;
mod chart;

use std::borrow::Cow;
use std::marker::PhantomData;

pub use button::*;
pub use chart::*;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{List, Paragraph, StatefulWidget, Tabs, Widget, WidgetRef};
use taffy::Size;

use crate::{MeasureNode, RenderNode};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Button,
    TextInput,
    Image,
    Text,
}

impl From<Role> for accesskit::Role {
    fn from(value: Role) -> Self {
        match value {
            Role::Button => accesskit::Role::Button,
            Role::TextInput => accesskit::Role::TextInput,
            Role::Image => accesskit::Role::Image,
            Role::Text => accesskit::Role::Label,
        }
    }
}

pub trait WidgetRole {
    fn widget_role() -> Option<Role>;
}

impl WidgetRole for () {
    fn widget_role() -> Option<Role> {
        None
    }
}

impl WidgetRole for String {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl WidgetRole for &'_ str {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl WidgetRole for Cow<'_, str> {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl WidgetRole for Text<'_> {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl WidgetRole for Line<'_> {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl WidgetRole for Span<'_> {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl MeasureNode for Tabs<'_> {
    fn measure(
        &self,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> taffy::Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl WidgetRole for Tabs<'_> {
    fn widget_role() -> Option<Role> {
        None
    }
}

impl MeasureNode for List<'_> {
    fn measure(
        &self,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl WidgetRole for List<'_> {
    fn widget_role() -> Option<Role> {
        None
    }
}

impl WidgetRole for Paragraph<'_> {
    fn widget_role() -> Option<Role> {
        Some(Role::Text)
    }
}

impl MeasureNode for ratatui::widgets::Sparkline<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl WidgetRole for ratatui::widgets::Sparkline<'_> {
    fn widget_role() -> Option<Role> {
        None
    }
}

pub trait RenderWidgetMark<M> {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

#[doc(hidden)]
pub struct __RefRender;

impl<T> RenderWidgetMark<__RefRender> for T
where
    for<'a> &'a T: Widget,
{
    fn render(&self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

#[doc(hidden)]
pub struct __OwnedRefRender;

impl<T> RenderWidgetMark<__OwnedRefRender> for T
where
    T: WidgetRef,
{
    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

pub struct RenderWidgetRef<W, M>
where
    W: RenderWidgetMark<M>,
{
    widget: W,
    _phantom: PhantomData<M>,
}

impl<W, M> RenderWidgetRef<W, M>
where
    W: RenderWidgetMark<M>,
{
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            _phantom: PhantomData,
        }
    }
}

impl<W, M> RenderNode for RenderWidgetRef<W, M>
where
    W: RenderWidgetMark<M>,
{
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        self.widget.render(rect, frame.buffer_mut())
    }
}

impl<W, M> WidgetRole for RenderWidgetRef<W, M>
where
    W: WidgetRole + RenderWidgetMark<M>,
{
    fn widget_role() -> Option<Role> {
        W::widget_role()
    }
}

impl<W, M> MeasureNode for RenderWidgetRef<W, M>
where
    W: MeasureNode + RenderWidgetMark<M>,
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

impl<W> WidgetRole for RenderWidget<W>
where
    W: Widget + WidgetRole,
{
    fn widget_role() -> Option<Role> {
        W::widget_role()
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

impl<W> WidgetRole for RenderStatefulWidget<W>
where
    W: StatefulWidget + Clone + WidgetRole,
{
    fn widget_role() -> Option<Role> {
        W::widget_role()
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

#[cfg(feature = "effects")]
impl MeasureNode for tachyonfx::widget::EffectTimeline {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> Size<f32> {
        self.estimate_size()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}
