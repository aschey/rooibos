use ratatui::prelude::*;
use ratatui::style::{Style, Styled};
use ratatui::symbols;
use ratatui::widgets::{Block, RenderDirection, Widget, WidgetRef};

#[derive(Clone, Default)]
pub struct Sparkline<'a> {
    inner: ratatui::widgets::Sparkline<'a>,
    data: Vec<u64>,
}

impl<'a> Sparkline<'a> {
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

impl<'a> WidgetRef for Sparkline<'a> {
    fn render_ref(&self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        self.inner.clone().data(&self.data).render(area, buf)
    }
}

impl<'a> Widget for Sparkline<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf)
    }
}

impl<'a> Styled for Sparkline<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style.into())
    }
}
