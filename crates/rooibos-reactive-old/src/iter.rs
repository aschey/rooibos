//! Reactive utilities for dealing with lists and iterables.

use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

use super::*;

/// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
/// computed, meaning that it's value will only be updated when requested. Modifications to the
/// input `Vec` are diffed using keys to prevent recomputing values that have not changed.
///
/// This function is the underlying utility behind `Keyed`.
///
/// # Params
/// * `list` - The list to be mapped. The list must be a [`ReadSignal`] (obtained from a [`Signal`])
///   and therefore reactive.
/// * `map_fn` - A closure that maps from the input type to the output type.
/// * `key_fn` - A closure that returns an _unique_ key to each entry.
///
///  _Credits: Based on TypeScript implementation in <https://github.com/solidjs/solid>_
pub fn map_keyed<T, K, U: 'static>(
    cx: Scope,
    list: impl Accessor<Vec<T>> + Clone + 'static,
    map_fn: impl Fn(Scope, T) -> U + 'static,
    key_fn: impl Fn(&T) -> K + 'static,
) -> Memo<Vec<U>>
where
    T: PartialEq + Clone + 'static,
    K: Eq + Hash,
    U: Clone,
{
    // Previous state used for diffing.
    let mut items = Vec::new();

    let mut mapped: Vec<U> = Vec::new();
    let mut mapped_tmp: Vec<Option<U>> = Vec::new();

    let mut disposers: Vec<Option<Scope>> = Vec::new();
    let mut disposers_tmp: Vec<Option<Scope>> = Vec::new();

    // Diff and update signal each time list is updated.
    create_memo(cx, move || {
        let new_items = list.value();
        if new_items.is_empty() {
            // Fast path for removing all items.
            for dis in mem::take(&mut disposers) {
                dis.unwrap().dispose();
            }
            mapped = Vec::new();
        } else if items.is_empty() {
            // Fast path for new create.
            mapped.reserve(new_items.len());
            disposers.reserve(new_items.len());

            for new_item in new_items.iter().cloned() {
                let map_fn = &map_fn;
                let mapped = &mut mapped;
                let child_cx = create_child_scope(cx);
                mapped.push(map_fn(child_cx, new_item));
                disposers.push(Some(child_cx));
            }
        } else {
            mapped_tmp.clear();
            mapped_tmp.resize(new_items.len(), None);

            disposers_tmp.clear();
            disposers_tmp.resize_with(new_items.len(), || None);

            // Skip common prefix.
            let min_len = usize::min(items.len(), new_items.len());
            let start = items
                .iter()
                .zip(new_items.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(min_len);
            debug_assert!(
                (items.get(start).is_none() && new_items.get(start).is_none())
                    || (items.get(start) != new_items.get(start)),
                "start is the first index where items[start] != new_items[start]"
            );

            // Skip common suffix.
            let mut end = items.len();
            let mut new_end = new_items.len();
            while end > start && new_end > start && items[end - 1] == new_items[new_end - 1] {
                end -= 1;
                new_end -= 1;
                mapped_tmp[new_end] = Some(mapped[end].clone());
                disposers_tmp[new_end] = disposers[end].take();
            }
            debug_assert!(
                if end != 0 && new_end != 0 {
                    (end == items.len() && new_end == new_items.len())
                        || (items[end - 1] != new_items[new_end - 1])
                } else {
                    true
                },
                "end and new_end are the last indexes where items[end - 1] != new_items[new_end - \
                 1]"
            );

            // 0) Prepare a map of indices in newItems. Scan backwards so we encounter them in
            // natural order.
            let mut new_indices = HashMap::with_capacity(new_end - start);

            // Indexes for new_indices_next are shifted by start because values at 0..start are
            // always None.
            let mut new_indices_next = vec![None; new_end - start];
            for j in (start..new_end).rev() {
                let item = &new_items[j];
                let i = new_indices.get(&key_fn(item));
                new_indices_next[j - start] = i.copied();
                new_indices.insert(key_fn(item), j);
            }

            // 1) Step through old items and see if they can be found in new set; if so, mark
            // them as moved.
            for i in start..end {
                let item = &items[i];
                if let Some(j) = new_indices.get(&key_fn(item)).copied() {
                    // Moved. j is index of item in new_items.
                    mapped_tmp[j] = Some(mapped[i].clone());
                    disposers_tmp[j] = disposers[i].take();
                    new_indices_next[j - start].and_then(|j| new_indices.insert(key_fn(item), j));
                } else {
                    // Create new.
                    disposers[i].take().unwrap().dispose();
                }
            }

            // 2) Set all the new values, pulling from the moved array if copied, otherwise
            // entering the new value.
            for j in start..new_items.len() {
                if matches!(mapped_tmp.get(j), Some(Some(_))) {
                    // Pull from moved array.
                    if j >= mapped.len() {
                        debug_assert_eq!(mapped.len(), j);
                        mapped.push(mapped_tmp[j].clone().unwrap());
                        disposers.push(disposers_tmp[j].take());
                    } else {
                        mapped[j] = mapped_tmp[j].clone().unwrap();
                        disposers[j] = disposers_tmp[j].take();
                    }
                } else {
                    // Create new value.

                    let new_item = new_items[j].clone();
                    let child_cx = create_child_scope(cx);
                    let tmp = map_fn(child_cx, new_item);
                    if mapped.len() > j {
                        mapped[j] = tmp;
                        disposers[j] = Some(child_cx);
                    } else {
                        mapped.push(tmp);
                        disposers.push(Some(child_cx));
                    }
                }
            }
        }

        // 3) In case the new set is shorter than the old, set the length of the mapped array.
        mapped.truncate(new_items.len());
        disposers.truncate(new_items.len());

        // 4) Save a copy of the mapped items for the next update.
        debug_assert!(
            [mapped.len(), disposers.len()]
                .iter()
                .all(|l| *l == new_items.len())
        );
        items = new_items;

        mapped.clone()
    })
}

/// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
/// computed, meaning that it's value will only be updated when requested. Modifications to the
/// input `Vec` are diffed by index to prevent recomputing values that have not changed.
///
/// Generally, it is preferred to use [`map_keyed`] instead when a key function
/// is available.
///
/// This function is the underlying utility behind `Indexed`.
///
/// # Params
/// * `list` - The list to be mapped. The list must be a [`ReadSignal`] (obtained from a [`Signal`])
///   and therefore reactive.
/// * `map_fn` - A closure that maps from the input type to the output type.
pub fn map_indexed<T, U: 'static>(
    cx: Scope,
    list: impl Accessor<Vec<T>> + Clone + 'static,
    map_fn: impl Fn(Scope, T) -> U + 'static,
) -> Memo<Vec<U>>
where
    T: PartialEq + Clone + 'static,
    U: Clone,
{
    // Previous state used for diffing.
    let mut items = Vec::new();
    let mut mapped = Vec::new();
    let mut disposers: Vec<Scope> = Vec::new();

    // Diff and update signal each time list is updated.
    create_memo(cx, move || {
        let new_items = list.value();

        if new_items.is_empty() {
            // Fast path for removing all items.
            for dis in mem::take(&mut disposers) {
                dis.dispose();
            }
            items = Vec::new();
            mapped = Vec::new();
        } else {
            // Pre-allocate space needed
            if new_items.len() > items.len() {
                let new_count = new_items.len() - items.len();
                mapped.reserve(new_count);
                disposers.reserve(new_count);
            }

            for (i, new_item) in new_items.iter().cloned().enumerate() {
                let item = items.get(i);
                // We lift the equality out of the else if branch to satisfy borrow checker.
                let eqs = item != Some(&new_item);

                if item.is_none() || eqs {
                    let child_cx = create_child_scope(cx);
                    let tmp = map_fn(cx, new_item);
                    if item.is_none() {
                        mapped.push(tmp);
                        disposers.push(child_cx);
                    } else if eqs {
                        mapped[i] = tmp;
                        let prev = mem::replace(&mut disposers[i], child_cx);
                        prev.dispose();
                    }
                }
            }

            if new_items.len() < items.len() {
                for _i in new_items.len()..items.len() {
                    disposers.pop().unwrap().dispose();
                }
            }

            // In case the new set is shorter than the old, set the length of the mapped array.
            mapped.truncate(new_items.len());

            // Save a copy of the mapped items for the next update.
            debug_assert!(
                [mapped.len(), disposers.len()]
                    .iter()
                    .all(|l| *l == new_items.len())
            );
            items = new_items;
        }

        // Update signal to trigger updates.
        mapped.clone()
    })
}
