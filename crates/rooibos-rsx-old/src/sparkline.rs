use ratatui::prelude::Rect;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{Block, RenderDirection, Sparkline, Widget};
use ratatui::{symbols, Frame};
use rooibos_reactive_old::Scope;

use crate::{MakeBuilder, View};

#[derive(Clone, Default)]
pub struct SparklineProps<'a> {
    inner: Sparkline<'a>,
    data: Vec<u64>,
}

impl<'a> SparklineProps<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn data(mut self, data: Vec<u64>) -> Self {
        self.data = data;
        self
    }

    pub fn max(mut self, max: u64) -> Self {
        self.inner = self.inner.max(max);
        self
    }

    pub fn bar_set(mut self, bar_set: symbols::bar::Set) -> Self {
        self.inner = self.inner.bar_set(bar_set);
        self
    }

    pub fn direction(mut self, direction: RenderDirection) -> Self {
        self.inner = self.inner.direction(direction);
        self
    }
}

impl<'a> Widget for SparklineProps<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        self.inner.data(&self.data).render(area, buf)
    }
}

impl<'a> Styled for SparklineProps<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style.into())
    }
}

impl MakeBuilder for SparklineProps<'_> {}

pub fn sparkline(_cx: Scope, props: SparklineProps<'static>) -> impl View {
    move |frame: &mut Frame, rect: Rect| frame.render_widget(props.clone(), rect)
}
