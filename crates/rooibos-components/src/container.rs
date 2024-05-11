use ratatui::layout::Constraint;
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{col, row, Constrainable, Render, RenderAny};

pub fn container<M>(
    h_constraint: impl Into<MaybeSignal<Constraint>>,
    v_constraint: impl Into<MaybeSignal<Constraint>>,
    children: M,
) -> impl Render
where
    M: RenderAny + 'static,
{
    row![col![children].constraint(h_constraint)].constraint(v_constraint)
}
