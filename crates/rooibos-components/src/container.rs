use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::prelude::*;

#[component]
pub fn Container<M>(
    #[prop(children)] children: M,
    #[prop(into)] h_constraint: MaybeSignal<Constraint>,
    #[prop(into)] v_constraint: MaybeSignal<Constraint>,
) -> impl Render
where
    M: RenderAny + 'static,
{
    view! {
        <row v:constraint=v_constraint>
            <col v:constraint=h_constraint>
                {children}
            </col>
        </row>
    }
}
