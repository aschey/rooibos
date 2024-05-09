use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeSignal, Signal};
use rooibos_dom::prelude::*;

use crate::{Show, ShowProps};
#[component]
fn PopupInner<M>(
    #[prop(children)] children: M,
    percent_x: MaybeSignal<u16>,
    percent_y: MaybeSignal<u16>,
    #[prop(default=None)] constraint: Option<Constraint>,
) -> impl Render
where
    M: RenderAny + 'static,
{
    let inverse_y = Signal::derive(move || (100 - percent_y.get()) / 2);
    let inverse_x = Signal::derive(move || (100 - percent_x.get()) / 2);

    view! {
        <col v:constraint=constraint.unwrap_or_default()>
            <row v:percentage=inverse_y.get() />
            <row v:percentage=percent_y.get()>
                <col v:percentage=inverse_x.get() />
                <col v:percentage=percent_x.get()>
                    <overlay>
                        <clear/>
                        {children}
                    </overlay>
                </col>
                <col v:percentage=inverse_x.get() />
            </row>
            <row v:percentage=inverse_y.get() />
        </col>
    }
}

#[component]
pub fn Popup<M, W, F>(
    #[prop(children)] mut children: F,
    visible: W,
    #[prop(into)] percent_x: MaybeSignal<u16>,
    #[prop(into)] percent_y: MaybeSignal<u16>,
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
