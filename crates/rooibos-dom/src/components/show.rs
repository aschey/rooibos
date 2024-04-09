use std::sync::Arc;

use either_of::Either;
use reactive_graph::computed::{ArcMemo, Memo};
use reactive_graph::traits::Get;
use tachys::view::Render;

use crate::prelude::*;

#[component]
pub fn Show<C, W, F, R1, R2>(
    #[prop(children)] mut children: C,
    when: W,
    fallback: F,
) -> impl Render<RooibosDom>
where
    C: Fn() -> R1 + 'static,
    F: Fn() -> R2 + 'static,
    W: Fn() -> bool + Send + Sync + 'static,
    R1: Render<RooibosDom> + 'static,
    R2: Render<RooibosDom> + 'static,
{
    let memoized_when = Memo::new(move |_| when());

    move || {
        if memoized_when.get() {
            Either::Left(children())
        } else {
            Either::Right(fallback())
        }
    }
}
