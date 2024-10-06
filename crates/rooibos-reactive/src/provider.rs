use reactive_graph::owner::{Owner, provide_context};
use tachys::reactive_graph::OwnedView;

use crate::RenderAny;

pub fn provider<T, F, R>(value: T, children: F) -> impl RenderAny
where
    T: Send + Sync + 'static,
    F: FnOnce() -> R,
    R: RenderAny + 'static,
{
    let owner = Owner::current()
        .expect("no current reactive Owner found")
        .child();
    let children = owner.with(|| {
        provide_context(value);
        (children)()
    });
    OwnedView::new_with_owner(children, owner)
}
