//! Callbacks define a standard way to store functions and closures. They are useful
//! for component properties, because they can be used to define optional callback functions,
//! which generic props don't support.
//!
//! # Usage
//! Callbacks can be created manually from any function or closure, but the easiest way
//! to create them is to use `#[prop(into)]]` when defining a component.
//! ```
//! # use rooibos::*;
//! #[component]
//! fn MyComponent(#[prop(into)] render_number: Callback<i32, String>) -> impl IntoView {
//!     view! {
//!         <div>
//!             {render_number.call(1)}
//!             // callbacks can be called multiple times
//!             {render_number.call(42)}
//!         </div>
//!     }
//! }
//! // you can pass a closure directly as `render_number`
//! fn test() -> impl IntoView {
//!     view! {
//!         <MyComponent render_number=|x: i32| x.to_string()/>
//!     }
//! }
//! ```
//!
//! *Notes*:
//! - The `render_number` prop can receive any type that implements `Fn(i32) -> String`.
//! - Callbacks are most useful when you want optional generic props.
//! - All callbacks implement the [`Callable`] trait, and can be invoked with
//!   `my_callback.call(input)`. On nightly, you can even do `my_callback(input)`
//! - The callback types implement [`Copy`], so they can easily be moved into and out of other
//!   closures, just like signals.
//!
//! # Types
//! This modules implements 2 callback types:
//! - [`Callback`]
//! - [`SyncCallback`]
//!
//! Use `SyncCallback` when you want the function to be `Sync` and `Send`.

use std::fmt;
use std::sync::Arc;

use crate::{store_value, StoredValue};

/// A wrapper trait for calling callbacks.
pub trait Callable<In: 'static, Out: 'static = ()> {
    /// calls the callback with the specified argument.
    fn call(&self, input: In) -> Out;
}

/// Callbacks define a standard way to store functions and closures.
///
/// # Example
/// ```
/// # use rooibos::*;
/// # use rooibos::{Callable, Callback};
/// #[component]
/// fn MyComponent(#[prop(into)] render_number: Callback<i32, String>) -> impl IntoView {
///     view! {
///         <div>
///             {render_number.call(42)}
///         </div>
///     }
/// }
///
/// fn test() -> impl IntoView {
///     view! {
///         <MyComponent render_number=move |x: i32| x.to_string()/>
///     }
/// }
/// ```

pub struct Callback<In: 'static, Out: 'static = ()>(StoredValue<Box<dyn Fn(In) -> Out>>);

impl<In> fmt::Debug for Callback<In> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str("Callback")
    }
}

impl<In, Out> Clone for Callback<In, Out> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<In, Out> Copy for Callback<In, Out> {}

impl<In, Out> Callback<In, Out> {
    /// Creates a new callback from the given function.
    pub fn new<F>(f: F) -> Callback<In, Out>
    where
        F: Fn(In) -> Out + 'static,
    {
        Self(store_value(Box::new(f)))
    }
}

impl<In: 'static, Out: 'static> Callable<In, Out> for Callback<In, Out> {
    fn call(&self, input: In) -> Out {
        self.0.with_value(|f| f(input))
    }
}

macro_rules! impl_from_fn {
    ($ty:ident) => {
        impl<F, In, T, Out> From<F> for $ty<In, Out>
        where
            F: Fn(In) -> T + 'static,
            T: Into<Out> + 'static,
        {
            fn from(f: F) -> Self {
                Self::new(move |x| f(x).into())
            }
        }
    };
}

impl_from_fn!(Callback);

/// A callback type that is `Send` and `Sync` if its input type is `Send` and `Sync`.
/// Otherwise, you can use exactly the way you use [`Callback`].
pub struct SyncCallback<In: 'static, Out: 'static = ()>(StoredValue<Arc<dyn Fn(In) -> Out>>);

impl<In> fmt::Debug for SyncCallback<In> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str("SyncCallback")
    }
}

impl<In, Out> Callable<In, Out> for SyncCallback<In, Out> {
    fn call(&self, input: In) -> Out {
        self.0.with_value(|f| f(input))
    }
}

impl<In, Out> Clone for SyncCallback<In, Out> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<In: 'static, Out: 'static> SyncCallback<In, Out> {
    /// Creates a new callback from the given function.
    pub fn new<F>(fun: F) -> Self
    where
        F: Fn(In) -> Out + 'static,
    {
        Self(store_value(Arc::new(fun)))
    }
}

impl_from_fn!(SyncCallback);

#[cfg(test)]
mod tests {
    use crate::callback::{Callback, SyncCallback};
    use crate::create_runtime;

    struct NoClone {}

    #[test]
    fn clone_callback() {
        let rt = create_runtime();
        let callback = Callback::new(move |_no_clone: NoClone| NoClone {});
        let _cloned = callback;
        rt.dispose();
    }

    #[test]
    fn clone_sync_callback() {
        let rt = create_runtime();
        let callback = SyncCallback::new(move |_no_clone: NoClone| NoClone {});
        let _cloned = callback.clone();
        rt.dispose();
    }

    #[test]
    fn callback_from() {
        let rt = create_runtime();
        let _callback: Callback<(), String> = (|()| "test").into();
        rt.dispose();
    }

    #[test]
    fn sync_callback_from() {
        let rt = create_runtime();
        let _callback: SyncCallback<(), String> = (|()| "test").into();
        rt.dispose();
    }
}
