//! Stores, an easy way to make complicated data reactive.

use std::cell::RefCell;
use std::collections::BTreeMap;

use super::{create_signal, Scope, Signal};

struct StoreState<T: 'static> {
    value: T,
    trie: RefCell<Option<PathTriggerTrie>>,
}

pub struct Store<T: 'static> {
    inner: Signal<StoreState<T>>,
    cx: Scope,
}

pub fn create_store<T>(cx: Scope, value: T) -> Store<T> {
    Store {
        inner: create_signal(
            cx,
            StoreState {
                value,
                trie: RefCell::new(Some(PathTriggerTrie {
                    trigger: create_signal(cx, ()),
                    children: BTreeMap::default(),
                })),
            },
        ),
        cx,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Path {
    segments: &'static [PathSegment],
}

impl Path {
    pub const EMPTY: Path = Path { segments: &[] };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathSegment {
    Ident(&'static str),
}

#[derive(Debug)]
struct PathTriggerTrie {
    trigger: Signal<()>,
    children: BTreeMap<PathSegment, PathTriggerTrie>,
}

impl PathTriggerTrie {
    fn track_path(&mut self, cx: Scope, path: Path) {
        self.trigger.0.track();
        if let [first, rest @ ..] = path.segments {
            self.children
                .entry(*first)
                .or_insert(PathTriggerTrie {
                    trigger: create_signal(cx, ()),
                    children: BTreeMap::default(),
                })
                .track_path(cx, Path { segments: rest })
        }
    }

    fn get_trigger_at_end_of_path(&mut self, cx: Scope, path: Path) -> Signal<()> {
        match path.segments {
            [first, rest @ ..] => self
                .children
                .entry(*first)
                .or_insert(PathTriggerTrie {
                    trigger: create_signal(cx, ()),
                    children: BTreeMap::default(),
                })
                .get_trigger_at_end_of_path(cx, Path { segments: rest }),
            [] => self.trigger,
        }
    }
}

impl<T> Store<T> {
    pub fn with_untracked<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.inner.0.with_untracked(|x| f(&x.value))
    }

    pub fn update_silent<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        self.inner.0.update_silent(|x| f(&mut x.value))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn get_inner_trie<U>(self, f: impl FnOnce(&mut PathTriggerTrie) -> U) -> U {
        let mut trie = self
            .inner
            .0
            .with(|x| x.trie.take())
            .expect("get_inner_trie should not be called inside itself");
        let ret = f(&mut trie);
        self.inner.0.with(|x| *x.trie.borrow_mut() = Some(trie));
        ret
    }

    #[doc(hidden)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn track_path(self, path: Path) {
        self.get_inner_trie(|trie| trie.track_path(self.cx, path));
    }

    #[doc(hidden)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn trigger_path(self, path: Path) {
        let trigger = self.get_inner_trie(|trie| trie.get_trigger_at_end_of_path(self.cx, path));
        trigger.0.set(());
    }
}

impl<T> Clone for Store<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Store<T> {}

/// Construct a [`Path`] from tokens.
#[macro_export]
#[doc(hidden)]
macro_rules! construct_path {
    ($(. $member:ident)*) => {
        Path {
            segments: &[$($crate::PathSegment::Ident(stringify!($member)),)*]
        }
    }
}

/// Read from a [`Store`].
#[macro_export]
macro_rules! read {
    ($var:ident $(. $path:tt)*) => {{
        let path = $crate::construct_path!($(. $path)*);
        $var.track_path(path);
        $var.with_untracked(|x| x $(. $path)* )
    }};
}

/// Write to a [`Store`].
#[macro_export]
macro_rules! set {
    ($var:ident = $value:expr) => {
        $var.update_silent(|x| *x = $value );
        $var.trigger_path($crate::Path::EMPTY);
    };
    ($var:ident $(. $path:tt)+ = $value:expr) => {
        let path = $crate::construct_path!($(. $path)*);
        $var.update_silent(|x| x $(. $path)* = $value );
        $var.trigger_path(path);
    };
}