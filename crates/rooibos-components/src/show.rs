use either_of::Either;
use reactive_graph::traits::Get;
use rooibos_dom::{IntoView, TypedChildrenMut, ViewFn};

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

    pub fn fallback(mut self, fallback: impl Into<ViewFn>) -> Self {
        self.fallback = fallback.into();
        self
    }

    pub fn render<C, W>(self, when: W, children: impl Into<TypedChildrenMut<C>>) -> impl IntoView
    where
        C: IntoView + 'static,
        W: Get<Value = bool> + Send + 'static,
    {
        let Self { fallback } = self;

        let mut children = children.into().into_inner();
        move || {
            if when.get() {
                Either::Left(children())
            } else {
                Either::Right(fallback.run())
            }
        }
    }
}
