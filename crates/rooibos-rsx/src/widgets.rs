use std::cell::RefCell;

use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Context};
use ratatui::widgets::{StatefulWidget, *};
use ratatui::Frame;
use rooibos_reactive::Scope;
use rooibos_rsx_macros::{impl_stateful_widget, impl_widget};

use crate::View;

pub trait Backend: ratatui::backend::Backend + 'static {}

impl<B> Backend for B where B: ratatui::backend::Backend + 'static {}

pub trait StatefulRender<B, W>
where
    B: Backend,
    W: StatefulWidget,
{
    fn render_with_state(&mut self, widget: W, frame: &mut Frame<B>, rect: Rect);
}

pub trait MakeBuilder {}

pub trait BuilderFacade {
    fn builder() -> Self;
}

pub trait BuildFacade {
    fn build(self) -> Self;
    fn __caller_id(self, caller_id: u64) -> Self;
}

impl<T> BuilderFacade for T
where
    T: MakeBuilder + Default,
{
    fn builder() -> Self {
        Self::default()
    }
}

impl<T> BuildFacade for T
where
    T: MakeBuilder,
{
    fn build(self) -> Self {
        self
    }

    fn __caller_id(self, _caller_id: u64) -> Self {
        self
    }
}

pub trait NewExt<'a, T>
where
    Self: 'a,
{
    fn new(source: T) -> Self;
}

pub trait NewFrom {}

impl<'a, S, T> NewExt<'a, T> for S
where
    S: NewFrom + 'a,
    Self: From<T>,
{
    fn new(source: T) -> Self {
        Self::from(source)
    }
}

impl<'a> NewFrom for Line<'a> {}
impl<'a> NewFrom for Span<'a> {}
impl<'a> NewFrom for Cell<'a> {}
impl<'a> NewFrom for Text<'a> {}

pub trait StyleExt<'a> {
    fn style(self, style: Style) -> Self;
}

impl<'a> StyleExt<'a> for Span<'a> {
    fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> StyleExt<'a> for Text<'a> {
    fn style(mut self, style: Style) -> Self {
        self.reset_style();
        self.patch_style(style);
        self
    }
}

impl<'a> MakeBuilder for Row<'a> {}
impl<'a> MakeBuilder for Cell<'a> {}
impl<'a> MakeBuilder for Span<'a> {}
impl<'a> MakeBuilder for ListItem<'a> {}
impl<'a> MakeBuilder for Line<'a> {}
impl<'a> MakeBuilder for Text<'a> {}
impl<'a> MakeBuilder for Axis<'a> {}
impl MakeBuilder for Style {}
impl MakeBuilder for ListState {}
impl MakeBuilder for TableState {}
impl MakeBuilder for Wrap {}

impl_widget!(Block, visibility=pub, generics=<'a>);
impl_widget!(Paragraph, visibility=pub, generics=<'a>);
impl_widget!(List, visibility=pub,generics=<'a>);
impl_widget!(Tabs, visibility=pub, generics=<'a>);
impl_widget!(Table, visibility=pub, generics=<'a>);
impl_widget!(Gauge, visibility=pub, generics=<'a>);
impl_widget!(LineGauge, visibility=pub, generics=<'a>);
impl_widget!(BarChart, visibility=pub, generics=<'a>);
impl_widget!(Clear, visibility=pub);
impl_widget!(Canvas, visibility=pub, generics=<'a, F>, where_clause=where F: Fn(&mut Context) + Clone + 'static);
impl_stateful_widget!(List, visibility=pub, generics=<'a>);
impl_stateful_widget!(Table, visibility=pub, generics=<'a>);
impl_stateful_widget!(Scrollbar, visibility=pub, generics=<'a>);

pub trait WrapExt {
    fn trim(self, trim: bool) -> Self;
}

impl WrapExt for Wrap {
    fn trim(self, trim: bool) -> Self {
        Self { trim }
    }
}
