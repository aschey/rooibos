//! Reactive signals.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

use slotmap::new_key_type;

use super::{store_value, EffectId, Root, Scope, StoredValue};

new_key_type! { pub(crate) struct SignalId; }

pub trait SignalGet<T> {
    /// Clones and returns the current value of the signal, and subscribes
    /// the running effect to this signal.
    ///
    /// # Panics
    /// Panics if you try to access a signal that is owned by a reactive node that has been
    /// disposed.
    #[track_caller]
    fn get(self) -> T;
}

pub trait SignalUpdate<T> {
    /// Applies a function to the current value to mutate it in place
    /// and notifies subscribers that the signal has changed.
    ///
    /// **Note:** `update()` does not auto-memoize, i.e., it will notify subscribers
    /// even if the value has not actually changed.
    #[track_caller]
    fn update<U>(self, f: impl FnOnce(&mut T) -> U) -> U;

    fn set(self, new: T) -> T;
}

/// Stores al the data associated with a signal.
pub(crate) struct SignalState {
    /// The value of the signal. This is wrapped inside an [`Option`] because this will allow us to
    /// temporarily take the value out while we run signal updates so that we do not have to hold
    /// on mutably to `root.signals`.
    pub value: RefCell<Option<Box<dyn Any>>>,
    /// List of signals whose value this signal depends on.
    ///
    /// If any of the dependency signals are updated, this signal will automatically be updated as
    /// well.
    pub dependencies: Vec<SignalId>,
    /// List of signals which depend on the value of this signal.
    ///
    /// If this signal updates, any dependent signal will automatically be updated as well.
    pub dependents: Vec<SignalId>,
    pub effect_dependents: Vec<EffectId>,
    /// A callback that automatically updates the value of the signal when one of its dependencies
    /// updates.
    ///
    /// A signal created using [`create_signal`] can be thought of as a signal which is never
    /// autoamtically updated. A signal created using [`create_memo`] can be thought of as a signal
    /// that is always automatically updated.
    ///
    /// Note that the update function takes a `&mut dyn Any`. The update function should only ever
    /// set this value to the same type as the signal.
    ///
    /// The return value of the update function is a `bool`. This should represent whether the
    /// value has been changed or not. If `true` is returned, then dependent signals will also be
    /// updated.
    pub update: Option<Box<dyn FnMut(&mut Box<dyn Any>) -> bool>>,
    /// An internal state used by `propagate_updates`. This should be `true` if the signal has been
    /// updated in the last call to `propagate_updates` and was reacheable from the start node.
    /// This is to stop propagation to dependents if this value is `false`.
    pub changed_in_last_update: bool,
    /// An internal state used by `propagate_updates`. This is used in DFS to detect cycles.
    pub mark: Mark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    /// Mark when DFS reaches node.
    Temp,
    /// Mark when DFS is done with node.
    Permanent,
    /// No mark.
    None,
}

pub(crate) enum ReadSignalType<T: 'static> {
    Base(BaseSignal<T>),
    Derived(StoredValue<Box<dyn Fn() -> T>>),
}

impl<T: fmt::Debug> fmt::Debug for ReadSignalType<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(base) => base.fmt(f),
            Self::Derived(derived) => derived.with_value(|v| v().fmt(f)),
        }
    }
}

impl<T: fmt::Display> fmt::Display for ReadSignalType<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(base) => base.fmt(f),
            Self::Derived(derived) => derived.with_value(|v| v().fmt(f)),
        }
    }
}

impl<T> Clone for ReadSignalType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignalType<T> {}

/// A read-only reactive value.
///
/// Unlike the difference between Rust's shared and mutable-references (`&T` and `&mut`), the
/// underlying data is not immutable. The data can be updated with the corresponding [`Signal`]
/// (which has mutable access) and will show up in the `ReadSignal` as well.
///
/// A `ReadSignal` can be simply obtained by dereferencing a [`Signal`]. In fact, every [`Signal`]
/// is a `ReadSignal` with additional write abilities!
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let signal: Signal<i32> = create_signal(cx, 123);
/// let read_signal: ReadSignal<i32> = *signal;
/// assert_eq!(read_signal.get(), 123);
/// signal.set(456);
/// assert_eq!(read_signal.get(), 456);
/// // read_signal.set(789); // <-- This is not allowed!
/// # });
/// ```
///
/// See [`create_signal`] for more information.
pub struct ReadSignal<T: 'static>(pub(crate) ReadSignalType<T>);

pub struct WriteSignal<T: 'static>(pub(crate) BaseSignal<T>);

pub struct Signal<T: 'static>(pub(crate) BaseSignal<T>);

/// A reactive value that can be read and written to.
///
/// This is the writable analog of [`ReadSignal`].
///
/// See [`create_signal`] for more information.
pub(crate) struct BaseSignal<T: 'static> {
    pub(crate) id: SignalId,
    root: &'static Root,
    _phantom: PhantomData<T>,
}

/// Create a new [`Signal`].
///
/// Signals are reactive atoms, pieces of state that can be read and written to and which will
/// automatically update anything which depend on them.
///
/// # Usage
/// The simplest way to use a signal is by using [`.get()`](ReadSignal::get) and
/// [`.set(...)`](Signal::set). However, this only works if the value implements [`Copy`]. If
/// we wanted to store something that doesn't implement [`Copy`] but implements [`Clone`] instead,
/// say a [`String`], we can use [`.get_clone()`](ReadSignal::get_clone) which will automatically
/// clone the value for us.
///
/// ```rust
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let signal = create_signal(cx, 1);
/// signal.get(); // Should return 1.
/// signal.set(2);
/// signal.get(); // Should return 2.
/// //
/// # });
/// ```
///
/// There are many other ways of getting and setting signals, such as
/// [`.with(...)`](ReadSignal::with) and [`.update(...)`](Signal::update) which can access the
/// signal even if it does not implement [`Clone`] or if you simply don't want to pay the
/// performance overhead of cloning your value everytime you read it.
///
/// # Reactivity
/// What makes signals so powerful, as opposed to some other wrapper type like [`RefCell`] is the
/// automatic dependency tracking. This means that accessing a signal will automatically add it as
/// a dependency in certain contexts (such as inside a [`create_memo`](crate::create_memo)) which
/// allows us to update related state whenever the signal is changed.
///
/// ```rust
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let signal = create_signal(cx, 1);
/// // Note that we are accessing signal inside a closure in the line below. This will cause it to
/// // be automatically tracked and update our double value whenever signal is changed.
/// let double = create_memo(cx, move || signal.get() * 2);
/// double.get(); // Should return 2.
/// signal.set(2);
/// double.get(); // Should return 4. Notice how this value was updated automatically when we
/// // modified signal. This way, we can rest assured that all our state will be
/// // consistent at all times!
/// # });
/// ```
///
/// # Ownership
/// Signals are always associated with a [`Scope`]. This is what performs the memory management for
/// the actual value of the signal. What is returned from this function is just a handle/reference
/// to the signal allocted in the [`Scope`]. This allows us to freely copy this handle around and
/// use it in closures and event handlers without worrying about ownership of the signal.
///
/// This is why in the above example, we could access `signal` even after it was moved in to the
/// closure of the `create_memo`.
#[cfg_attr(debug_assertions, track_caller)]
pub fn create_signal<T>(cx: Scope, value: T) -> Signal<T> {
    let data = SignalState {
        value: RefCell::new(Some(Box::new(value))),
        dependencies: Vec::new(),
        effect_dependents: Vec::new(),
        dependents: Vec::new(),
        update: None,
        changed_in_last_update: false,
        mark: Mark::None,
    };
    let key = cx.root.signals.borrow_mut().insert(data);
    // Add the signal the scope signal list so that it is properly dropped when the scope is
    // dropped.
    cx.get_data(|cx| cx.signals.push(key));
    Signal(BaseSignal {
        id: key,
        root: cx.root,
        _phantom: PhantomData,
    })
}

impl<T> BaseSignal<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data<U>(self, f: impl FnOnce(&SignalState) -> U) -> U {
        f(self
            .root
            .signals
            .borrow()
            .get(self.id)
            .expect("signal is disposed"))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn get_data_mut<U>(self, f: impl FnOnce(&mut SignalState) -> U) -> U {
        f(self
            .root
            .signals
            .borrow_mut()
            .get_mut(self.id)
            .expect("signal is disposed"))
    }

    /// Get the value of the signal without tracking it. The type is [`Clone`]-ed automatically.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_untracked(self) -> T
    where
        T: Clone,
    {
        self.with_untracked(Clone::clone)
    }

    /// Get a value from the signal without tracking it.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn with_untracked<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.get_data(|signal| {
            f(signal
                .value
                .borrow()
                .as_ref()
                .expect("cannot get value while updating")
                .downcast_ref()
                .expect("wrong signal type in slotmap"))
        })
    }

    /// Get a value from the signal.
    ///
    /// When called inside a reactive scope, the signal will be automatically tracked.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn with<U>(self, f: impl FnOnce(&T) -> U) -> U {
        self.track();
        self.with_untracked(f)
    }

    /// Track the signal in the current reactive scope. This is done automatically when calling
    /// [`ReadSignal::get`] and other similar methods.
    pub fn track(self) {
        if let Some(tracker) = &mut *self.root.tracker.borrow_mut() {
            tracker.dependencies.push(self.id);
        }
    }
}

impl<T> ReadSignal<T> {
    pub fn derive(cx: Scope, derived_signal: impl Fn() -> T + 'static) -> Self {
        ReadSignal(ReadSignalType::Derived(store_value(
            cx,
            Box::new(derived_signal),
        )))
    }
}

pub trait IntoSignal<T>: Sized {
    /// Consumes `self`, returning a [`Signal<T>`].
    fn derive_signal(self, cx: Scope) -> ReadSignal<T>;
}

impl<F, T> IntoSignal<T> for F
where
    F: Fn() -> T + 'static,
{
    fn derive_signal(self, cx: Scope) -> ReadSignal<T> {
        ReadSignal::derive(cx, self)
    }
}

impl<T: Clone> SignalGet<T> for ReadSignal<T> {
    /// Get the value of the signal. The type must implement [`Copy`]. If this is not the case, use
    /// [`ReadSignal::get_clone_untracked`] or [`ReadSignal::with_untracked`] instead.
    ///
    /// When called inside a reactive scope, the signal will be automatically tracked.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive3::*;
    /// # create_root(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(state.get(), 1);
    ///
    /// // The signal is automatically tracked in the line below.
    /// let doubled = create_memo(cx, move || state.get());
    /// # });
    /// ```
    #[cfg_attr(debug_assertions, track_caller)]
    fn get(self) -> T {
        match self.0 {
            ReadSignalType::Base(base) => base.get(),
            ReadSignalType::Derived(derived) => derived.with_value(|f| f()),
        }
    }
}

impl<T: Clone> SignalGet<T> for Signal<T> {
    /// Get the value of the signal. The type must implement [`Copy`]. If this is not the case, use
    /// [`ReadSignal::get_clone_untracked`] or [`ReadSignal::with_untracked`] instead.
    ///
    /// When called inside a reactive scope, the signal will be automatically tracked.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive3::*;
    /// # create_root(|cx| {
    /// let state = create_signal(cx, 0);
    /// assert_eq!(state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(state.get(), 1);
    ///
    /// // The signal is automatically tracked in the line below.
    /// let doubled = create_memo(cx, move || state.get());
    /// # });
    /// ```
    #[cfg_attr(debug_assertions, track_caller)]
    fn get(self) -> T {
        self.0.get()
    }
}

impl<T> BaseSignal<T> {
    /// Update the value of the signal silently. This will not trigger any updates in dependent
    /// signals. As such, this is generally not recommended as it can easily lead to state
    /// inconsistencies.
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn update_silent<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        self.get_data(|signal| {
            f(signal
                .value
                .borrow_mut()
                .as_mut()
                .expect("cannot update while updating")
                .downcast_mut()
                .expect("wrong signal type in slotmap"))
        })
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn get(self) -> T
    where
        T: Clone,
    {
        self.track();
        self.get_untracked()
    }

    /// Update the value of the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    fn update<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        let ret = self.update_silent(f);
        self.root.propagate_updates(self.id);
        ret
    }

    /// Set a new value for the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn set(self, new: T) -> T {
        self.update(|val| std::mem::replace(val, new))
    }
}

impl<T> SignalUpdate<T> for WriteSignal<T> {
    /// Update the value of the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    fn update<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        self.0.update(f)
    }

    /// Set a new value for the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    fn set(self, new: T) -> T {
        self.0.set(new)
    }
}

impl<T> SignalUpdate<T> for Signal<T> {
    /// Update the value of the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    fn update<U>(self, f: impl FnOnce(&mut T) -> U) -> U {
        self.0.update(f)
    }

    /// Set a new value for the signal and automatically update any dependents.
    #[cfg_attr(debug_assertions, track_caller)]
    fn set(self, new: T) -> T {
        self.0.set(new)
    }
}

impl<T> Signal<T> {
    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (self.to_read_signal(), self.to_write_signal())
    }

    pub fn to_read_signal(self) -> ReadSignal<T> {
        ReadSignal(ReadSignalType::Base(self.0))
    }

    pub fn to_write_signal(self) -> WriteSignal<T> {
        WriteSignal(self.0)
    }
}

impl<F, T> SignalGet<T> for F
where
    F: Fn() -> T + 'static,
{
    fn get(self) -> T {
        self()
    }
}

/// We manually implement `Clone` + `Copy` for `Signal` so that we don't get extra bounds on `T`.
impl<T> Clone for BaseSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for BaseSignal<T> {}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Signal<T> {}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ReadSignal<T> {}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for WriteSignal<T> {}

// Formatting implementations for `ReadSignal` and `Signal`.
impl<T: fmt::Debug> fmt::Debug for BaseSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}

impl<T: fmt::Display> fmt::Display for BaseSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with(|value| value.fmt(f))
    }
}

impl<T: fmt::Debug> fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Debug> fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for WriteSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: AddAssign<Rhs>, Rhs> AddAssign<Rhs> for WriteSignal<T> {
    fn add_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this += rhs);
    }
}

impl<T: SubAssign<Rhs>, Rhs> SubAssign<Rhs> for WriteSignal<T> {
    fn sub_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this -= rhs);
    }
}

impl<T: MulAssign<Rhs>, Rhs> MulAssign<Rhs> for WriteSignal<T> {
    fn mul_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this *= rhs);
    }
}

impl<T: DivAssign<Rhs>, Rhs> DivAssign<Rhs> for WriteSignal<T> {
    fn div_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this /= rhs);
    }
}

impl<T: RemAssign<Rhs>, Rhs> RemAssign<Rhs> for WriteSignal<T> {
    fn rem_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this %= rhs);
    }
}

impl<T: AddAssign<Rhs>, Rhs> AddAssign<Rhs> for Signal<T> {
    fn add_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this += rhs);
    }
}

impl<T: SubAssign<Rhs>, Rhs> SubAssign<Rhs> for Signal<T> {
    fn sub_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this -= rhs);
    }
}

impl<T: MulAssign<Rhs>, Rhs> MulAssign<Rhs> for Signal<T> {
    fn mul_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this *= rhs);
    }
}

impl<T: DivAssign<Rhs>, Rhs> DivAssign<Rhs> for Signal<T> {
    fn div_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this /= rhs);
    }
}

impl<T: RemAssign<Rhs>, Rhs> RemAssign<Rhs> for Signal<T> {
    fn rem_assign(&mut self, rhs: Rhs) {
        self.update(|this| *this %= rhs);
    }
}
