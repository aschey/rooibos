mod chart;
mod sparkline;

pub use chart::*;
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Context};
use ratatui::widgets::*;
use rooibos_dom::prelude::*;
use rooibos_dom_macros::{impl_stateful_widget, impl_widget, make_builder};
pub use sparkline::*;

use crate::DomWidget;

#[make_builder]
pub trait MakeBuilder {}

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
    fn style(self, style: Style) -> Self {
        self.reset_style().patch_style(style)
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

impl_widget!(Block, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Paragraph, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(List, visibility=pub,generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Tabs, name_override=TabHeaders, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Table, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Gauge, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(LineGauge, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(BarChart, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Clear, visibility=pub, make_builder=MakeBuilder);
impl_widget!(Canvas, visibility=pub, generics=<'a, F>, make_builder=MakeBuilder, where_clause=where F: Fn(&mut Context) + Clone + 'static);
impl_stateful_widget!(List, visibility=pub, generics=<'a>);
impl_stateful_widget!(Table, visibility=pub, generics=<'a>);
impl_stateful_widget!(Scrollbar, visibility=pub, generics=<'a>, render_ref=false);

pub fn make_dom_widget<W: Widget + Clone + 'static>(
    name: impl Into<String>,
    widget: W,
) -> DomWidget {
    DomWidget::new(name, move || {
        let widget = widget.clone();
        move |frame, rect| {
            frame.render_widget(widget.clone(), rect);
        }
    })
}

pub trait WrapExt {
    fn trim(self, trim: bool) -> Self;
}

impl WrapExt for Wrap {
    fn trim(self, trim: bool) -> Self {
        Self { trim }
    }
}
