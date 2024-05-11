use ratatui::layout::Constraint;
use ratatui::widgets::Clear;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeSignal, Signal};
use rooibos_dom::{
    col, overlay, row, signal, widget_ref, Constrainable, IntoView, Render, RenderAny,
};

use crate::Show;

fn popup_inner<M>(
    percent_x: MaybeSignal<u16>,
    percent_y: MaybeSignal<u16>,
    constraint: Option<MaybeSignal<Constraint>>,
    children: M,
) -> impl Render
where
    M: RenderAny + 'static,
{
    let inverse_y = signal!((100 - percent_y.get()) / 2);
    let inverse_x = signal!((100 - percent_x.get()) / 2);

    col![
        row![].percentage(inverse_y),
        row![
            col![].percentage(inverse_x),
            col![overlay![widget_ref!(Clear), children]].percentage(percent_x)
        ]
        .percentage(percent_y),
    ]
    .constraint(constraint.unwrap_or_default().get())
}

#[derive(Default)]
pub struct Popup {
    percent_x: MaybeSignal<u16>,
    percent_y: MaybeSignal<u16>,
    constraint: Option<MaybeSignal<Constraint>>,
}

impl Popup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn percent_x(mut self, val: impl Into<MaybeSignal<u16>>) -> Self {
        self.percent_x = val.into();
        self
    }

    pub fn percent_y(mut self, val: impl Into<MaybeSignal<u16>>) -> Self {
        self.percent_y = val.into();
        self
    }

    pub fn render<M, W, F>(self, visible: W, mut children: F) -> impl IntoView
    where
        F: FnMut() -> M + Send + Sync + 'static,
        M: RenderAny + Send + Sync + 'static,
        W: Fn() -> bool + Send + Sync + 'static,
    {
        let Popup {
            percent_x,
            percent_y,
            constraint,
        } = self;

        Show::new().render(visible, move || {
            let children = children();
            popup_inner(percent_x, percent_y, constraint, children)
        })
    }
}

impl Constrainable for Popup {
    fn constraint<S>(mut self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        self.constraint = Some(constraint.into());
        self
    }
}
