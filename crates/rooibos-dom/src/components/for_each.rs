use std::hash::Hash;

use reactive_graph::owner::Owner;
use rooibos_dom_macros::component;
use tachys::reactive_graph::OwnedView;
use tachys::view::keyed::keyed;
use tachys::view::Render;

use crate::prelude::*;
// use crate::IntoView;

#[component]
pub fn ForEach<IF, I, T, EF, N, KF, K>(each: IF, key: KF, children: EF) -> impl Render<RooibosDom>
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(T) -> N + Clone + 'static,
    N: Render<RooibosDom> + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
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
    let children = move |child| {
        let owner = parent.with(Owner::new);
        let view = owner.with(|| children(child));
        OwnedView::new_with_owner(view, owner)
    };
    move || keyed(each(), key.clone(), children.clone())
}
