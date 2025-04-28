use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Widget, WidgetRef};

use super::{Role, WidgetRole};
use crate::MeasureNode;

pub struct Button<'a> {
    text: Text<'a>,
}

impl<'a> Button<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self { text: text.into() }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.text = self.text.alignment(alignment);
        self
    }

    pub fn left_aligned(self) -> Self {
        self.alignment(Alignment::Left)
    }

    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    pub fn right_aligned(self) -> Self {
        self.alignment(Alignment::Right)
    }
}

impl WidgetRef for Button<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.text.render_ref(area, buf);
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRole for Button<'_> {
    fn widget_role() -> Option<Role> {
        Some(Role::Button)
    }
}

impl MeasureNode for Button<'_> {
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        self.text.measure(known_dimensions, available_space, style)
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        self.text.estimate_size()
    }
}
