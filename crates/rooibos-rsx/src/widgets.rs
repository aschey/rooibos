use std::cell::RefCell;

use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Context};
use ratatui::widgets::{StatefulWidget, *};
use ratatui::Frame;
use rooibos_reactive::Scope;

use crate::View;

macro_rules! impl_widget {
    ($name:ident, $widget:ident, $props:ident) => {
        pub type $props<'a> = $widget<'a>;

        impl MakeBuilder for $props<'_> {}

        pub fn $name<B: Backend>(_cx: Scope, props: $props<'static>) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(props.clone(), rect)
        }
    };
}

macro_rules! impl_widget_no_lifetime {
    ($name:ident, $widget:ident, $props:ident) => {
        pub type $props = $widget;

        impl MakeBuilder for $props {}

        pub fn $name<B: Backend>(_cx: Scope, props: $props) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(props.clone(), rect)
        }
    };
}

macro_rules! impl_stateful_widget {
    ($name:ident, $widget:ident, $props:ident, $state:ident) => {
        impl<'a, B> StatefulRender<B, $props<'a>> for RefCell<$state>
        where
            B: Backend,
        {
            fn render_with_state(&mut self, widget: $props, frame: &mut Frame<B>, rect: Rect) {
                frame.render_stateful_widget(widget, rect, &mut self.borrow_mut())
            }
        }

        impl<'a, B> StatefulRender<B, $props<'a>> for $state
        where
            B: Backend,
        {
            fn render_with_state(&mut self, widget: $props, frame: &mut Frame<B>, rect: Rect) {
                frame.render_stateful_widget(widget, rect, &mut self.clone())
            }
        }

        pub type $props<'a> = $widget<'a>;

        pub fn $name<'a, B: Backend>(
            _cx: Scope,
            props: $props<'static>,
            mut state: impl StatefulRender<B, $widget<'a>> + 'static,
        ) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| {
                state.render_with_state(props.clone(), frame, rect);
            }
        }
    };
}

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

impl_widget!(block, Block, BlockProps);
impl_widget!(paragraph, Paragraph, ParagraphProps);
impl_widget!(list, List, ListProps);
impl_widget!(tabs, Tabs, TabsProps);
impl_widget!(table, Table, TableProps);
impl_widget!(gauge, Gauge, GaugeProps);
impl_widget!(line_gauge, LineGauge, LineGaugeProps);
impl_widget!(bar_chart, BarChart, BarChartProps);
impl_widget_no_lifetime!(clear, Clear, ClearProps);
impl_stateful_widget!(stateful_list, List, StatefulListProps, ListState);
impl_stateful_widget!(stateful_table, Table, StatefulTableProps, TableState);
impl_stateful_widget!(
    stateful_scrollbar,
    Scrollbar,
    StatefulScrollbarProps,
    ScrollbarState
);

pub trait WrapExt {
    fn trim(self, trim: bool) -> Self;
}

impl WrapExt for Wrap {
    fn trim(self, trim: bool) -> Self {
        Self { trim }
    }
}

pub type CanvasProps<'a, F> = Canvas<'a, F>;

impl<F> MakeBuilder for CanvasProps<'_, F> where F: Fn(&mut Context) {}

pub fn canvas<B: Backend, F>(_cx: Scope, props: CanvasProps<'static, F>) -> impl View<B>
where
    F: Fn(&mut Context) + Clone + 'static,
{
    move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(props.clone(), rect)
}
