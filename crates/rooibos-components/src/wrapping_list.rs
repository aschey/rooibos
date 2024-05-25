use std::ops::{Deref, DerefMut};

pub trait Keyed {
    type Key: Eq;
    fn key(&self) -> &Self::Key;
}

#[derive(Clone)]
pub struct KeyedWrappingList<T>(pub Vec<T>);

impl<T> Deref for KeyedWrappingList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for KeyedWrappingList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> KeyedWrappingList<T>
where
    T: Keyed,
{
    pub fn next_item(&self, focused: &T::Key) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let current = self.iter().position(|t| t.key() == focused).unwrap();
        if current == self.len() - 1 {
            self.first()
        } else {
            Some(&self[current + 1])
        }
    }

    pub fn prev_item(&self, focused: &T::Key) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let current = self.iter().position(|t| t.key() == focused).unwrap();
        if current == 0 {
            self.last()
        } else {
            Some(&self[current - 1])
        }
    }
}

#[derive(Clone)]
pub struct WrappingList<T>(pub Vec<T>);

impl<T> Deref for WrappingList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WrappingList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> WrappingList<T> {
    pub fn next_index(&self, focused_index: usize) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        if focused_index == self.len() - 1 {
            Some(0)
        } else {
            Some(focused_index + 1)
        }
    }

    pub fn prev_index(&self, focused_index: usize) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        if focused_index == 0 {
            Some(self.len() - 1)
        } else {
            Some(focused_index - 1)
        }
    }
}
