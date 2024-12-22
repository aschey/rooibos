use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Block, GraphType, WidgetRef};
use style::Styled;
use taffy::AvailableSpace;

use crate::MeasureNode;

#[derive(Clone, Default)]
pub struct Dataset<'a> {
    inner: ratatui::widgets::Dataset<'a>,
    data: Vec<(f64, f64)>,
}

impl<'a> Dataset<'a> {
    pub fn name<S>(mut self, name: S) -> Self
    where
        S: Into<Line<'a>>,
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

impl Styled for Dataset<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        Self {
            inner: self.inner.set_style(style),
            data: self.data,
        }
    }
}

#[derive(Clone)]
pub struct Chart<'a> {
    datasets: Vec<Dataset<'a>>,
    block: Option<Block<'a>>,
    x_axis: Axis<'a>,
    y_axis: Axis<'a>,
    style: Style,
    hidden_legend_constraints: (Constraint, Constraint),
}

impl<'a> Chart<'a> {
    pub fn new(datasets: Vec<Dataset<'a>>) -> Self {
        Self {
            block: None,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            style: Style::default(),
            datasets,
            hidden_legend_constraints: (Ratio(1, 4), Ratio(1, 4)),
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

impl Styled for Chart<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style.into())
    }
}

impl WidgetRef for Chart<'_> {
    fn render_ref(&self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut chart = ratatui::widgets::Chart::new(
            self.datasets
                .iter()
                .map(|d| d.inner.clone().data(&d.data))
                .collect(),
        )
        .style(self.style)
        .x_axis(self.x_axis.clone())
        .y_axis(self.y_axis.clone())
        .hidden_legend_constraints(self.hidden_legend_constraints);
        if let Some(block) = self.block.clone() {
            chart = chart.block(block);
        }

        chart.render(area, buf)
    }
}

impl Widget for Chart<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        self.render_ref(area, buf)
    }
}

impl MeasureNode for Chart<'_> {
    fn measure(
        &self,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<AvailableSpace>,
        _style: &taffy::Style,
    ) -> taffy::Size<f32> {
        taffy::Size::zero()
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        taffy::Size::zero()
    }
}
