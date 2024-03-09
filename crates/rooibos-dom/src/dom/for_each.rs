use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};
use rooibos_reactive::{as_child_of_current_owner, create_render_effect, on_cleanup, Disposer};
use rustc_hash::FxHasher;

use super::unmount_child;
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
    mounted: Rc<OnceCell<DomNode>>,
    opening: DomNode,
    pub(crate) children: Rc<RefCell<Vec<EachItem>>>,
    closing: DomNode,
}

impl EachRepr {
    pub(crate) fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}

impl Default for EachRepr {
    fn default() -> Self {
        let markers = (DomNode::transparent("Each"), DomNode::transparent("Each"));

        let document_fragment = DocumentFragment::transparent("each");

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            mounted: Default::default(),
            opening: markers.0,
            children: Default::default(),
            closing: markers.1,
            name: "each".to_owned(),
        }
    }
}

impl Mountable for EachRepr {
    fn get_mountable_node(&self) -> DomNode {
        if let Some(mounted) = self.mounted.get() {
            mounted.clone()
        } else {
            mount_child(MountKind::Append(&self.document_fragment), &self.opening);
            mount_child(MountKind::Append(&self.document_fragment), &self.closing);
            self.mounted.set(self.document_fragment.clone()).unwrap();
            self.document_fragment.clone()
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct EachItem {
    disposer: Disposer,
    document_fragment: DomNode,
    opening: DomNode,
    pub(crate) child: View,
    closing: DomNode,
    mounted: Rc<OnceCell<DomNode>>,
}

impl EachItem {
    fn new(disposer: Disposer, child: View) -> Self {
        let document_fragment = DocumentFragment::transparent("each");
        let markers = (
            DomNode::transparent("EachItem"),
            DomNode::transparent("EachItem"),
        );

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            opening: markers.0,
            closing: markers.1,
            child,
            disposer,
            mounted: Default::default(),
        }
    }
}

impl Mountable for EachItem {
    fn get_mountable_node(&self) -> DomNode {
        if let Some(mounted) = self.mounted.get() {
            mounted.clone()
        } else {
            mount_child(MountKind::Append(&self.document_fragment), &self.opening);
            mount_child(MountKind::Append(&self.document_fragment), &self.closing);
            mount_child(
                MountKind::Before(&self.closing),
                &self.child.get_mountable_node(),
            );
            self.mounted.set(self.document_fragment.clone()).unwrap();
            self.document_fragment.clone()
        }
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

        let opening = component.opening.clone();
        let children = component.children.clone();
        let closing = component.closing.clone();

        let each_fn = as_child_of_current_owner(each_fn);

        create_render_effect(move |prev_hash_run: Option<HashRun<FxIndexSet<K>>>| {
            let mut children_borrow = children.borrow_mut();

            // let opening = if let Some(child) = children_borrow.first() {
            //     child.get_opening_node()
            // } else {
            //     closing.clone()
            // };

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
                    unmount_child(current_item.document_fragment.key());
                }

                for (idx, item) in items.into_iter().enumerate() {
                    if !prev_hash_run.contains(&hashed_items[idx]) {
                        let (child, disposer) = each_fn(item);
                        let each_item = EachItem::new(disposer, child.into_view());

                        // fragment.append_child(&each_item.get_mountable_node());

                        children_borrow.push(each_item);
                    } else {
                        let prev_index = prev_hash_run.get_index_of(&hashed_items[idx]).unwrap();
                        let new_item = current_items[prev_index].take().unwrap();
                        mount_child(MountKind::Append(&opening), &new_item.get_mountable_node());
                        children_borrow.push(current_items[prev_index].take().unwrap());
                    }
                }

                return HashRun(hashed_items);
            }
            *children_borrow = Vec::with_capacity(capacity);
            let fragment = DomNode::from_fragment(DocumentFragment::transparent("each"));

            for item in items_iter {
                hashed_items.insert(key_fn(&item));
                let (child, disposer) = each_fn(item);
                let each_item = EachItem::new(disposer, child.into_view());

                fragment.append_child(&each_item.get_mountable_node());

                children_borrow.push(each_item);
            }
            mount_child(MountKind::Before(&closing), &fragment);

            HashRun(hashed_items)
        });

        View::Each(component)
    }
}
