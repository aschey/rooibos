use std::hash::Hash;

use reactive_graph::owner::Owner;
use reactive_graph::signal::{ArcRwSignal, ReadSignal};
use reactive_graph::traits::Set;
use rooibos_dom::IntoView;
use tachys::reactive_graph::OwnedView;
use tachys::view::keyed::keyed;

pub fn for_each<IF, I, T, EF, N, KF, K>(each: IF, key: KF, children: EF) -> impl IntoView
where
    IF: Fn() -> I + Send + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(T) -> N + Clone + Send + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Clone + Send + 'static,
    K: Eq + Hash + 'static,
    T: 'static,
{
    // this takes the owner of the For itself
    // this will end up with N + 1 children
    // 1) the effect for the `move || keyed(...)` updates
    // 2) an owner for each child
    //
    // this means
    // a) the reactive owner for each row will not be cleared when the whole list updates
    // b) context provided in each row will not wipe out the others
    let parent = Owner::current().expect("no reactive owner");
    let children = move |_, child| {
        let owner = parent.with(Owner::new);
        let view = owner.with(|| children(child));
        (|_| {}, OwnedView::new_with_owner(view, owner))
    };
    move || keyed(each(), key.clone(), children.clone())
}

pub fn for_enumerate<IF, I, T, EF, N, KF, K>(each: IF, key: KF, children: EF) -> impl IntoView
where
    IF: Fn() -> I + Send + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(ReadSignal<usize>, T) -> N + Send + Clone + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Clone + Send + 'static,
    K: Eq + Hash + 'static,
    T: 'static,
{
    // this takes the owner of the For itself
    // this will end up with N + 1 children
    // 1) the effect for the `move || keyed(...)` updates
    // 2) an owner for each child
    //
    // this means
    // a) the reactive owner for each row will not be cleared when the whole list updates
    // b) context provided in each row will not wipe out the others
    let parent = Owner::current().expect("no reactive owner");
    let children = move |index, child| {
        let owner = parent.with(Owner::new);
        let (index, set_index) = ArcRwSignal::new(index).split();
        let view = owner.with(|| children(index.into(), child));
        (
            move |index| set_index.set(index),
            OwnedView::new_with_owner(view, owner),
        )
    };
    move || keyed(each(), key.clone(), children.clone())
}
