use ratatui::layout::Constraint;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{
    col, constraint, derive_signal, percentage, props, row, Constrainable, IntoView, Render,
    RenderAny,
};

use crate::Show;

fn popup_inner<M>(
    percent_x: MaybeSignal<u16>,
    percent_y: MaybeSignal<u16>,
    constraint_: Option<MaybeSignal<Constraint>>,
    children: M,
) -> impl Render
where
    M: RenderAny + 'static,
{
    let inverse_y = derive_signal!((100 - percent_y.get()) / 2);
    let inverse_x = derive_signal!((100 - percent_x.get()) / 2);

    col![
        props!(constraint(constraint_.unwrap_or_default().get()));
        row![props!(percentage(inverse_y));],
        row![
            props!(percentage(percent_y));
            col![props!(percentage(inverse_x));],
            col![children].percentage(percent_x),
            col![props!(percentage(inverse_x));],
        ],
        row![props!(percentage(inverse_y));]
    ]
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
        W: Get<Value = bool> + Send + Sync + 'static,
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
    type Output = Self;

    fn constraint<S>(mut self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        self.constraint = Some(constraint.into());
        self
    }
}
