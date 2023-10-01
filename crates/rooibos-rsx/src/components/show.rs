use rooibos_reactive::Scope;

use crate::prelude::*;

#[component]
pub fn Show<F1, F2, V1, V2, W>(
    _cx: Scope,
    #[prop(children)] children: F1,
    when: W,
    fallback: F2,
    #[prop(default = true)] lazy: bool,
) -> impl View
where
    W: Fn() -> bool + 'static,
    F1: Fn() -> V1 + 'static,
    F2: Fn() -> V2 + 'static,
    V1: View,
    V2: View,
{
    move || match (when(), lazy) {
        (true, false) => {
            fallback();
            children().into_boxed_view()
        }
        (true, true) => children().into_boxed_view(),
        (false, false) => {
            children().into_boxed_view();
            fallback().into_boxed_view()
        }
        (false, true) => fallback().into_boxed_view(),
    }
}
