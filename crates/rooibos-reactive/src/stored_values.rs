use std::cell::RefCell;
use std::fmt::{self, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

use super::{Root, Scope};

slotmap::new_key_type! {
    /// Unique ID assigned to a [`StoredValue`].
    pub(crate) struct StoredValueId;
}

pub struct StoredValue<T>
where
    T: 'static,
{
    id: StoredValueId,
    root: &'static Root,
    _phantom: PhantomData<T>,
}

impl<T> StoredValue<T> {
    pub fn with_value<U>(self, f: impl FnOnce(&T) -> U) -> U {
        f(self
            .root
            .stored_values
            .borrow()
            .get(self.id)
            .expect("value is disposed")
            .borrow()
            .downcast_ref::<T>()
            .unwrap())
    }

    pub fn get_value(self) -> T
    where
        T: Clone,
    {
        self.with_value(T::clone)
    }

    pub fn update_value<O>(self, f: impl FnOnce(&mut T) -> O) -> O {
        f(self
            .root
            .stored_values
            .borrow()
            .get(self.id)
            .expect("value is disposed")
            .borrow_mut()
            .downcast_mut::<T>()
            .unwrap())
    }

    pub fn set_value(self, new: T) -> T {
        self.update_value(|val| std::mem::replace(val, new))
    }
}

pub fn store_value<T>(cx: Scope, value: T) -> StoredValue<T>
where
    T: 'static,
{
    let key = cx
        .root
        .stored_values
        .borrow_mut()
        .insert(Rc::new(RefCell::new(value)));
    StoredValue {
        id: key,
        root: cx.root,
        _phantom: PhantomData,
    }
}

impl<T> Copy for StoredValue<T> {}

impl<T> Clone for StoredValue<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: fmt::Debug> fmt::Debug for StoredValue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with_value(|value| value.fmt(f))
    }
}

impl<T: fmt::Display> fmt::Display for StoredValue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with_value(|value| value.fmt(f))
    }
}
