use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};
use rooibos_reactive::{as_child_of_current_owner, create_render_effect, on_cleanup, Disposer};
use rustc_hash::FxHasher;

use super::{disconnect_child, unmount_child};
use crate::dom::mount_child;
use crate::{DocumentFragment, DomNode, IntoView, MountKind, Mountable, View};

type FxIndexSet<T> = indexmap::IndexSet<T, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

struct HashRun<T>(T);

impl<T> fmt::Debug for HashRun<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_tuple("HashRun").finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct EachRepr {
    name: String,
    document_fragment: DomNode,
    children: Rc<RefCell<Vec<EachItem>>>,
}

impl EachRepr {
    pub(crate) fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}

impl Default for EachRepr {
    fn default() -> Self {
        let document_fragment = DocumentFragment::transparent("each");

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            children: Default::default(),
            name: "each".to_owned(),
        }
    }
}

impl Mountable for EachRepr {
    fn get_mountable_node(&self) -> DomNode {
        self.document_fragment.clone()
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct EachItem {
    disposer: Disposer,
    child: View,
}

impl EachItem {
    fn new(disposer: Disposer, child: View) -> Self {
        Self { child, disposer }
    }
}

impl Mountable for EachItem {
    fn get_mountable_node(&self) -> DomNode {
        self.child.get_mountable_node()
    }
}

pub struct Each<IF, I, T, EF, N, KF, K>
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(T) -> N + 'static,
    N: IntoView,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
    T: 'static,
{
    pub(crate) items_fn: IF,
    pub(crate) each_fn: EF,
    key_fn: KF,
}

impl<IF, I, T, EF, N, KF, K> Each<IF, I, T, EF, N, KF, K>
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(T) -> N + 'static,
    N: IntoView,
    KF: Fn(&T) -> K,
    K: Eq + Hash + 'static,
    T: 'static,
{
    pub const fn new(items_fn: IF, key_fn: KF, each_fn: EF) -> Self {
        Self {
            items_fn,
            each_fn,
            key_fn,
        }
    }
}

impl<IF, I, T, EF, N, KF, K> IntoView for Each<IF, I, T, EF, N, KF, K>
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(T) -> N + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
    T: 'static,
{
    fn into_view(self) -> View {
        let Self {
            items_fn,
            each_fn,
            key_fn,
        } = self;

        let component = EachRepr::default();

        let children = component.children.clone();
        let fragment = component.document_fragment.clone();

        let each_fn = as_child_of_current_owner(each_fn);

        create_render_effect(move |prev_hash_run: Option<HashRun<FxIndexSet<K>>>| {
            let mut children_borrow = children.borrow_mut();

            let items_iter = items_fn().into_iter();

            let (capacity, _) = items_iter.size_hint();
            let mut hashed_items =
                FxIndexSet::with_capacity_and_hasher(capacity, Default::default());

            if let Some(HashRun(prev_hash_run)) = prev_hash_run {
                let mut items = Vec::with_capacity(capacity);
                for item in items_iter {
                    hashed_items.insert(key_fn(&item));
                    items.push(item);
                }

                let removed = prev_hash_run.difference(&hashed_items);
                for removed_item in removed {
                    let idx = prev_hash_run.get_index_of(removed_item).unwrap();
                    let removed_node = children_borrow[idx].get_mountable_node();
                    unmount_child(removed_node.key());
                }
                let mut current_items: Vec<_> = children_borrow.drain(..).map(Some).collect();

                for current_item in current_items.iter().flatten() {
                    disconnect_child(current_item.get_mountable_node().key());
                }

                for (idx, item) in items.into_iter().enumerate() {
                    let item = if !prev_hash_run.contains(&hashed_items[idx]) {
                        let (child, disposer) = each_fn(item);
                        EachItem::new(disposer, child.into_view())
                    } else {
                        let prev_index = prev_hash_run.get_index_of(&hashed_items[idx]).unwrap();
                        current_items[prev_index].take().unwrap()
                    };
                    mount_child(MountKind::Append(&fragment), &item.get_mountable_node());
                    children_borrow.push(item);
                }

                return HashRun(hashed_items);
            }
            *children_borrow = Vec::with_capacity(capacity);

            for item in items_iter {
                hashed_items.insert(key_fn(&item));
                let (child, disposer) = each_fn(item);
                let each_item = EachItem::new(disposer, child.into_view());

                fragment.append_child(&each_item.get_mountable_node());

                children_borrow.push(each_item);
            }

            HashRun(hashed_items)
        });

        View::Each(component)
    }
}
