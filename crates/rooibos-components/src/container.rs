use rooibos_dom::prelude::*;

#[component]
pub fn Container<M>(
    #[prop(children)] children: M,
    h_constraint: Constraint,
    v_constraint: Constraint,
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
