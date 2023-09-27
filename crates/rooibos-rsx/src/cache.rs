use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use rooibos_reactive::{create_child_scope, Scope, StoredValue};

use crate::View;

pub struct KeyData {
    pub cx: Scope,
    pub stored_view: StoredValue<Rc<RefCell<dyn View>>>,
    pub iteration: u32,
}

pub struct WidgetCache {
    pub view_cache: RefCell<HashMap<(u64, u64), KeyData>>,
    pub scope_cache: ScopeCache,
    iteration: AtomicU32,
}

impl WidgetCache {
    pub fn next_iteration(&self) {
        self.iteration.fetch_add(1, Ordering::SeqCst);
    }

    pub fn mark(&self, node: &mut KeyData) {
        let iter = self.iteration.load(Ordering::SeqCst);
        node.iteration = iter;
    }

    pub fn evict(&self) {
        let mut cache_mut = self.view_cache.borrow_mut();
        let current_iteration = self.iteration.load(Ordering::SeqCst);

        for val in cache_mut.values() {
            if val.iteration < current_iteration && !val.cx.is_disposed() && !val.cx.is_root() {
                val.cx.dispose();
            }
        }

        let keys: Vec<_> = cache_mut.keys().copied().collect();
        for k in &keys {
            if let Some(val) = cache_mut.get(k) {
                if val.cx.is_disposed() {
                    cache_mut.remove(k);
                }
            }
        }

        self.scope_cache.evict();
    }
}

#[derive(Default)]
pub struct ScopeCache {
    scopes: Rc<RefCell<HashMap<(u64, u64, Option<String>), Scope>>>,
}

impl ScopeCache {
    pub fn get_or_create<K>(&self, cx: Scope, caller_id: u64, key: Option<K>) -> Scope
    where
        K: Display,
    {
        let mut scopes = self.scopes.borrow_mut();
        if let Some(child_cx) =
            scopes.get(&(cx.id(), caller_id, key.as_ref().map(|k| k.to_string())))
        {
            *child_cx
        } else {
            let child_cx = create_child_scope(cx);
            scopes.insert((cx.id(), caller_id, key.map(|k| k.to_string())), child_cx);
            child_cx
        }
    }

    fn evict(&self) {
        let mut scopes = self.scopes.borrow_mut();
        let keys: Vec<_> = scopes.keys().cloned().collect();
        for k in keys {
            if let Some(val) = scopes.get(&k) {
                if val.is_disposed() {
                    scopes.remove(&k);
                }
            }
        }
    }
}

thread_local! {
    pub static __WIDGET_CACHE: WidgetCache = WidgetCache {
        view_cache: Default::default(),
        scope_cache: ScopeCache::default(),
        iteration: AtomicU32::new(0)
    };

}
