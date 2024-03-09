use std::hash::Hash;

use rooibos_dom_macros::component;

use crate::prelude::*;
use crate::IntoView;

#[component]
pub fn ForEach<IF, I, T, EF, N, KF, K>(each: IF, key: KF, children: EF) -> impl IntoView
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(T) -> N + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
    T: 'static,
{
    Each::new(each, key, children).into_view()
}
