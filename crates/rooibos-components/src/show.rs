use either_of::Either;
use reactive_graph::computed::Memo;
use reactive_graph::traits::Get;
use rooibos_dom::prelude::*;

pub struct Show {
    fallback: ViewFn,
}

impl Default for Show {
    fn default() -> Self {
        Self::new()
    }
}

impl Show {
    pub fn new() -> Self {
        Self {
            fallback: (|| {}).into(),
        }
    }

    pub fn render<C, W>(self, when: W, children: impl Into<TypedChildrenMut<C>>) -> impl IntoView
    where
        C: IntoView + 'static,
        W: Fn() -> bool + Send + Sync + 'static,
    {
        let Self { fallback } = self;
        let memoized_when = Memo::new(move |_| when());

        let mut children = children.into().into_inner();
        move || {
            if memoized_when.get() {
                Either::Left(children())
            } else {
                Either::Right(fallback.run())
            }
        }
    }
}
