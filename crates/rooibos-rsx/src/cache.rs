use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use rooibos_reactive::{
    create_child_scope, create_signal, ReadSignal, Scope, Signal, SignalUpdate, StoredValue,
};

use crate::View;

pub struct KeyData {
    pub cx: Scope,
    pub stored_view: StoredValue<Rc<RefCell<dyn View>>>,
    pub iteration: u32,
}

pub struct WidgetCache {
    pub view_cache: RefCell<HashMap<(u64, u64), KeyData>>,
    pub scope_cache: ScopeCache,
    focusable_scopes: RefCell<HashSet<u64>>,
    active_scopes: RefCell<Vec<Scope>>,
    focused_index: RefCell<Option<usize>>,
    iteration: AtomicU32,
    focused: RefCell<Option<Signal<Option<Scope>>>>,
}

impl WidgetCache {
    pub fn next_iteration(&self) {
        self.active_scopes.borrow_mut().clear();
        self.focusable_scopes.borrow_mut().clear();
        self.iteration.fetch_add(1, Ordering::SeqCst);
    }

    pub fn mark(&self, node: &mut KeyData) {
        let iter = self.iteration.load(Ordering::SeqCst);
        node.iteration = iter;
        self.active_scopes.borrow_mut().push(node.cx);
    }

    pub fn init_root(&self, root: Scope) {
        *self.focused.borrow_mut() = Some(create_signal(root, None));
    }

    pub fn focused_scope(&self) -> ReadSignal<Option<Scope>> {
        self.focused.borrow().unwrap().to_read_signal()
    }

    pub fn focus_next(&self) {
        let mut focused_index = self.focused_index.borrow().map(|i| i + 1).unwrap_or(0);
        let (focused, focused_index) = loop {
            let new_cx = self.active_scopes.borrow()[focused_index];
            if self.focusable_scopes.borrow().contains(&new_cx.id()) {
                break (new_cx, focused_index);
            }
            focused_index += 1;
        };
        self.focused.borrow_mut().unwrap().set(Some(focused));
        *self.focused_index.borrow_mut() = Some(focused_index);
    }

    pub fn get_or_create<K>(
        &self,
        cx: Scope,
        focusable: bool,
        caller_id: u64,
        key: Option<K>,
    ) -> Scope
    where
        K: Display,
    {
        let new_scope = self.scope_cache.get_or_create(cx, caller_id, key);
        if focusable {
            self.focusable_scopes.borrow_mut().insert(new_scope.id());
        }
        new_scope
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
    fn get_or_create<K>(&self, cx: Scope, caller_id: u64, key: Option<K>) -> Scope
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
        scope_cache: Default::default(),
        focusable_scopes: Default::default(),
        focused: Default::default(),
        active_scopes: Default::default(),
        iteration: AtomicU32::new(0),
        focused_index: Default::default()
    };

}
