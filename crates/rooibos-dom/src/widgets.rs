use std::borrow::Cow;
use std::sync::atomic::{AtomicU32, Ordering};

use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Context};
use ratatui::widgets::{StatefulWidget, *};
use ratatui::Frame;
use rooibos_dom_macros::{impl_stateful_render, impl_stateful_widget, impl_widget, make_builder};

use crate::{DomWidget, IntoView, View};

pub static __NODE_ID: AtomicU32 = AtomicU32::new(1);

#[make_builder]
pub trait MakeBuilder {}

impl_stateful_render!(StatefulRender, visibility=pub);

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
impl_widget!(Tabs, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Table, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Gauge, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(LineGauge, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(BarChart, visibility=pub, generics=<'a>, make_builder=MakeBuilder);
impl_widget!(Clear, visibility=pub, make_builder=MakeBuilder);
impl_widget!(Canvas, visibility=pub, generics=<'a, F>, make_builder=MakeBuilder, where_clause=where F: Fn(&mut Context) + Clone + 'static);
impl_stateful_widget!(List, visibility=pub, generics=<'a>, stateful_render=StatefulRender);
impl_stateful_widget!(Table, visibility=pub, generics=<'a>, stateful_render=StatefulRender);
impl_stateful_widget!(Scrollbar, visibility=pub, generics=<'a>, stateful_render=StatefulRender);

pub fn make_dom_widget<W: Widget + Clone + 'static>(
    name: impl Into<String>,
    widget: W,
) -> DomWidget {
    DomWidget::new(
        __NODE_ID.fetch_add(1, Ordering::Relaxed),
        name,
        move |frame, rect| {
            #[cfg(debug_assertions)]
            let prev = rooibos_reactive::SpecialNonReactiveZone::enter();
            frame.render_widget(widget.clone(), rect);
            #[cfg(debug_assertions)]
            rooibos_reactive::SpecialNonReactiveZone::exit(prev);
        },
    )
}

impl IntoView for String {
    fn into_view(self) -> View {
        make_dom_widget("String", self).into_view()
    }
}

impl IntoView for &'static str {
    fn into_view(self) -> View {
        make_dom_widget("String", self).into_view().into_view()
    }
}

impl<'a> IntoView for Cow<'a, str> {
    fn into_view(self) -> View {
        make_dom_widget("String", self.to_string()).into_view()
    }
}

pub trait WrapExt {
    fn trim(self, trim: bool) -> Self;
}

impl WrapExt for Wrap {
    fn trim(self, trim: bool) -> Self {
        Self { trim }
    }
}
