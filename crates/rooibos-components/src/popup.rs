use rooibos_dom::prelude::*;

use crate::{Show, ShowProps};
#[component]
fn PopupInner<M>(
    #[prop(children)] children: M,
    percent_x: u16,
    percent_y: u16,
    #[prop(default=None)] constraint: Option<Constraint>,
) -> impl Render
where
    M: RenderAny + 'static,
{
    let inverse_y = (100 - percent_y) / 2;
    let inverse_x = (100 - percent_x) / 2;

    view! {
        <col v:constraint=constraint.unwrap_or_default()>
            <row v:percentage=inverse_y />
            <row v:percentage=percent_y>
                <col v:percentage=inverse_x />
                <col v:percentage=percent_x>
                    <overlay>
                        <clear/>
                        {children}
                    </overlay>
                </col>
                <col v:percentage=inverse_x />
            </row>
            <row v:percentage=inverse_y />
        </col>
    }
}

#[component]
pub fn Popup<M, W, F>(
    #[prop(children)] mut children: F,
    visible: W,
    percent_x: u16,
    percent_y: u16,
    #[prop(default=None)] constraint: Option<Constraint>,
) -> impl IntoView
where
    F: FnMut() -> M + Send + 'static,
    M: RenderAny + 'static,
    W: Fn() -> bool + Send + Sync + 'static,
{
    view! {
        <Show
            when=visible
        >
            {move || {
                let children = children();
                view! {
                    <PopupInner percent_x=percent_x percent_y=percent_y constraint=constraint>
                        {children}
                    </PopupInner>
                }
            }}
       </Show>
    }
}
