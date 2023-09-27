use std::borrow::Cow;

use ratatui::prelude::{Constraint, Rect};
use ratatui::style::{Style, Styled};
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, Widget};
use ratatui::{symbols, Frame};
use rooibos_reactive::Scope;

use crate::widgets::MakeBuilder;
use crate::View;

#[derive(Clone, Default)]
pub struct DatasetOwned<'a> {
    inner: Dataset<'a>,
    data: Vec<(f64, f64)>,
}

impl<'a> DatasetOwned<'a> {
    pub fn name<S>(mut self, name: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.inner = self.inner.name(name);
        self
    }

    pub fn data<D>(mut self, data: D) -> Self
    where
        D: Into<Vec<(f64, f64)>>,
    {
        self.data = data.into();
        self
    }

    pub fn marker(mut self, marker: symbols::Marker) -> Self {
        self.inner = self.inner.marker(marker);
        self
    }

    pub fn graph_type(mut self, graph_type: GraphType) -> Self {
        self.inner = self.inner.graph_type(graph_type);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }
}

impl<'a> MakeBuilder for DatasetOwned<'a> {}

impl<'a> Styled for DatasetOwned<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style(self, style: Style) -> Self::Item {
        Self {
            inner: self.inner.set_style(style),
            data: self.data,
        }
    }
}

#[derive(Clone)]
pub struct ChartProps<'a> {
    datasets: Vec<DatasetOwned<'a>>,
    block: Option<Block<'a>>,
    x_axis: Axis<'a>,
    y_axis: Axis<'a>,
    style: Style,
    hidden_legend_constraints: (Constraint, Constraint),
}

impl<'a> ChartProps<'a> {
    pub fn new(datasets: Vec<DatasetOwned<'a>>) -> Self {
        Self {
            block: None,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            style: Style::default(),
            datasets,
            hidden_legend_constraints: (Constraint::Ratio(1, 4), Constraint::Ratio(1, 4)),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis(mut self, axis: Axis<'a>) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis<'a>) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn hidden_legend_constraints(mut self, constraints: (Constraint, Constraint)) -> Self {
        self.hidden_legend_constraints = constraints;
        self
    }
}

impl<'a> Styled for ChartProps<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}

impl<'a> MakeBuilder for ChartProps<'a> {}

impl<'a> Widget for ChartProps<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut chart = Chart::new(
            self.datasets
                .iter()
                .map(|d| d.inner.clone().data(&d.data))
                .collect(),
        )
        .style(self.style)
        .x_axis(self.x_axis)
        .y_axis(self.y_axis)
        .hidden_legend_constraints(self.hidden_legend_constraints);
        if let Some(block) = self.block {
            chart = chart.block(block);
        }

        chart.render(area, buf)
    }
}

pub fn chart(_cx: Scope, props: ChartProps<'static>) -> impl View {
    move |frame: &mut Frame, rect: Rect| frame.render_widget(props.clone(), rect)
}
