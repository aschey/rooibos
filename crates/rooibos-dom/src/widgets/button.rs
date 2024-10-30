use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Styled};
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Widget, WidgetRef};

use crate::MeasureNode;

pub struct Button<'a> {
    text: Text<'a>,
    inner: Paragraph<'a>,
}

impl<'a> Button<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        let text = text.into();
        Self {
            text: text.clone(),
            inner: Paragraph::new(text).centered(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.inner = self.inner.alignment(alignment);
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

impl<'a> Styled for Button<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> WidgetRef for Button<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.inner.render_ref(area, buf);
    }
}

impl<'a> Widget for Button<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
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
}
