//! Stores: easy nested recursive data.

use crate::{store_value, Scope, StoredValue};

pub struct Store<T: State + 'static> {
    value: StoredValue<T>,
    trigger: T::Trigger,
}

impl<T: State> Store<T> {
    /// Internal method for implementing the `get!` macro.
    #[doc(hidden)]
    pub fn __with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.value.with_value(f)
    }

    #[doc(hidden)]
    pub fn __update(&self, f: impl FnOnce(T) -> T) {
        self.value.update_value(f)
    }

    /// Internal method for implementing the `get!` macro.
    #[doc(hidden)]
    pub fn __trigger(&self) -> &T::Trigger {
        &self.trigger
    }
}

impl<T: State> Copy for Store<T> {}

impl<T: State> Clone for Store<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn create_store<T: State>(cx: Scope, value: T) -> Store<T> {
    let stored_value = store_value(cx, value);
    Store {
        value: stored_value,
        trigger: T::Trigger::new(cx),
    }
}

pub trait State: Clone {
    /// The type of the struct containing all the triggers for fine-grained reactivity.
    type Trigger: StateTrigger;
}

pub trait StateTrigger: Copy {
    fn new(cx: Scope) -> Self;
}

#[cfg(test)]
mod tests {
    use rooibos_reactive_macros::{get, set, State};

    use super::*;
    use crate::create_root;
    use crate::signals::{SignalGet, SignalUpdate};

    #[test]
    fn test_derive() {
        #[derive(Clone, State)]
        struct Foo {
            value: i32,
        }

        create_root(|cx| {
            let foo = create_store(cx, Foo { value: 123 });
            set!(foo.value, 456);
            assert_eq!(get!(foo.value), 456)
        });
    }
}
