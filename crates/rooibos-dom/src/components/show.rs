use std::sync::Arc;

use either_of::Either;
use reactive_graph::computed::ArcMemo;
use reactive_graph::traits::Get;
use tachys::view::any_view::AnyView;
use tachys::view::Render;

use crate::prelude::*;

type ChildrenFn = Arc<dyn Fn() -> AnyView<RooibosDom>>;

struct ViewFn(Arc<dyn Fn() -> AnyView<RooibosDom> + Send + Sync + 'static>);

impl ViewFn {
    /// Execute the wrapped function
    pub fn run(&self) -> AnyView<RooibosDom> {
        (self.0)()
    }
}

// #[component]
pub fn Show<W>(
    // #[prop(children, into)]
    children: ChildrenFn,
    when: W,
    // #[prop(optional, into)]
    fallback: ViewFn,
) -> impl Render<RooibosDom>
where
    W: Fn() -> bool + Send + Sync + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());

    move || {
        if memoized_when.get() {
            Either::Left(children())
        } else {
            Either::Right(fallback.run())
        }
    }
}
