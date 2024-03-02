use rooibos_reactive::{create_memo, SignalGet};

use crate::prelude::*;

#[component]
pub fn Show<W>(
    #[prop(children, into)] children: ViewFn,
    when: W,
    #[prop(optional, into)] fallback: ViewFn,
) -> impl IntoView
where
    W: Fn() -> bool + 'static,
{
    let memoized_when = create_memo(move |_| when());

    move || match memoized_when.get() {
        true => children.run().into_view(),
        false => fallback.run(),
    }
}
