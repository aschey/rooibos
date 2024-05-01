use either_of::Either;
use reactive_graph::computed::Memo;
use reactive_graph::traits::Get;

use crate::prelude::*;

#[component]
pub fn Show<C, W>(
    /// The children will be shown whenever the condition in the `when` closure returns `true`.
    #[prop(children)]
    mut children: C,
    /// A closure that returns a bool that determines whether this thing runs
    when: W,
    /// A closure that returns what gets rendered if the when statement is false. By default this
    /// is the empty view.
    #[prop(optional, into)]
    fallback: ViewFn,
) -> impl RenderAny
where
    C: IntoChildrenMut,
    W: Fn() -> bool + Send + Sync + 'static,
{
    let memoized_when = Memo::new(move |_| when());
    let mut children = children.into_children_mut();
    move || {
        if memoized_when.get() {
            Either::Left((children)())
        } else {
            Either::Right(fallback.run())
        }
    }
}
