//! Memos (aka. eager derived signals).

use super::{create_signal, DependencyTracker, ReadSignal, Scope, SignalGet, SignalUpdate};
use std::cell::RefCell;
use std::fmt::{self, Formatter};

/// A memoized derived signal.
///
/// Usually created using [`create_memo`], [`create_selector`], and [`create_selector_with`].
pub struct Memo<T: 'static>(pub(crate) ReadSignal<T>);

impl<T: Clone> SignalGet<T> for Memo<T> {
    fn get(self) -> T {
        self.0.get()
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Memo<T> {}

impl<T: fmt::Debug> fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: fmt::Display> fmt::Display for Memo<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
/// Create a new [`Signal`] from an initial value, an initial list of dependencies, and an update
/// function. Used in the implementation of [`create_memo`] and friends.
fn create_updated_signal<T>(
    cx: Scope,
    initial: T,
    initial_deps: DependencyTracker,
    mut f: impl FnMut(&mut T) -> bool + 'static,
) -> ReadSignal<T> {
    let signal = create_signal(cx, initial);
    initial_deps.create_signal_dependency_links(cx.root, signal.0.id);

    // Set the signal update callback as f.
    signal.0.get_data_mut(move |data| {
        data.update = Some(Box::new(move |any| {
            f(any.downcast_mut().expect("could not downcast memo value"))
        }))
    });
    let (read_signal, _) = signal.split();
    read_signal
}

/// Creates a memoized computation from some signals.
/// The output is derived from all the signals that are used within the memo closure.
/// If any of the tracked signals are updated, the memo is also updated.
///
/// # Difference from derived signals
///
/// Derived signals (functions referencing signals) are lazy and do not keep track of the result
/// of the computation. This means that the computation will not be executed until needed.
/// This also means that calling the derived signal twice will result in the same computation
/// twice.
///
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
/// let double = || state.get() * 2;
///
/// let _ = double();
/// // Here, the closure named double is called again.
/// // If the computation is expensive enough, this would be wasted work!
/// let _ = double();
/// # });
/// ```
///
/// Memos, on the other hand, are eagerly evaluated and will only run the computation when one
/// of its dependencies change.
///
/// Memos also incur a slightly higher performance penalty than simple derived signals, so unless
/// there is some computation involved, it will likely be faster to just use a derived signal.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 0);
/// let double = create_memo(cx, move || state.get() * 2);
///
/// assert_eq!(double.get(), 0);
/// state.set(1);
/// assert_eq!(double.get(), 2);
/// # });
/// ```
pub fn create_memo<T>(cx: Scope, mut f: impl FnMut() -> T + 'static) -> Memo<T> {
    let (initial, tracker) = cx.root.tracked_scope(&mut f);
    let signal = create_updated_signal(cx, initial, tracker, move |value| {
        *value = f();
        true
    });

    Memo(signal)
}

/// Creates a memoized value from some signals.
/// Unlike [`create_memo`], this function will not notify dependents of a
/// change if the output is the same.
///
/// It takes a comparison function to compare the old and new value, which returns `true` if
/// they are the same and `false` otherwise.
///
/// To use the type's [`PartialEq`] implementation instead of a custom function, use
/// [`create_selector`].
pub fn create_selector_with<T>(
    cx: Scope,
    mut f: impl FnMut() -> T + 'static,
    mut eq: impl FnMut(&T, &T) -> bool + 'static,
) -> Memo<T> {
    let (initial, tracker) = cx.root.tracked_scope(&mut f);
    let signal = create_updated_signal(cx, initial, tracker, move |value| {
        let new = f();
        if eq(&new, value) {
            false
        } else {
            *value = new;
            true
        }
    });

    Memo(signal)
}

/// Creates a memoized value from some signals.
/// Unlike [`create_memo`], this function will not notify dependents of a change if the output is the
/// same. That is why the output of the function must implement [`PartialEq`].
///
/// To specify a custom comparison function, use [`create_selector_with`].
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// # create_root(|cx| {
/// let state = create_signal(cx, 1);
/// let squared = create_selector(cx, move || state.get() * state.get());
/// assert_eq!(squared.get(), 1);
///
/// create_effect(cx, move || println!("x^2 = {}", squared.get()));
///
/// state.set(2); // Triggers the effect.
/// assert_eq!(squared.get(), 4);
///
/// state.set(-2); // Does not trigger the effect.
/// assert_eq!(squared.get(), 4);
/// # });
/// ```
pub fn create_selector<T>(cx: Scope, f: impl FnMut() -> T + 'static) -> Memo<T>
where
    T: PartialEq,
{
    create_selector_with(cx, f, PartialEq::eq)
}

/// An alternative to [`create_signal`] that uses a reducer to get the next
/// value.
///
/// It uses a reducer function that takes the previous value and a message and returns the next
/// value.
///
/// Returns a [`Memo`] and a dispatch function to send messages to the reducer.
///
/// # Params
/// * `initial` - The initial value of the state.
/// * `reducer` - A function that takes the previous value and a message and returns the next value.
///
/// # Example
/// ```
/// # use sycamore_reactive3::*;
/// enum Msg {
///     Increment,
///     Decrement,
/// }
///
/// # create_root(|cx| {
/// let (state, dispatch) = create_reducer(cx, 0, |&state, msg: Msg| match msg {
///     Msg::Increment => state + 1,
///     Msg::Decrement => state - 1,
/// });
///
/// assert_eq!(state.get(), 0);
/// dispatch(Msg::Increment);
/// assert_eq!(state.get(), 1);
/// dispatch(Msg::Decrement);
/// assert_eq!(state.get(), 0);
/// # });
/// ```
pub fn create_reducer<T, Msg>(
    cx: Scope,
    initial: T,
    reduce: impl FnMut(&T, Msg) -> T,
) -> (Memo<T>, impl Fn(Msg)) {
    let reduce = RefCell::new(reduce);
    let signal = create_signal(cx, initial);
    let (read_signal, write_signal) = signal.split();
    let dispatch = move |msg| write_signal.update(|value| *value = reduce.borrow_mut()(value, msg));
    (Memo(read_signal), dispatch)
}
