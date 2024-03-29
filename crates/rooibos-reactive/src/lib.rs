//! The reactive system for the [rooibos](https://docs.rs/rooibos/latest/rooibos/) Web framework.
//!
//! ## Fine-Grained Reactivity
//!
//! rooibos is built on a fine-grained reactive system, which means that individual reactive values
//! ("signals," sometimes known as observables) trigger the code that reacts to them ("effects,"
//! sometimes known as observers) to re-run. These two halves of the reactive system are
//! inter-dependent. Without effects, signals can change within the reactive system but never be
//! observed in a way that interacts with the outside world. Without signals, effects run once but
//! never again, as there's no observable value to subscribe to.
//!
//! Here are the most commonly-used functions and types you'll need to build a reactive system:
//!
//! ### Signals
//! 1. *Signals:* [`create_signal`], which returns a ([`ReadSignal`], [`WriteSignal`] tuple, or
//!    [`create_rw_signal`], which returns a signal [`RwSignal`] without this read-write
//!    segregation.
//! 2. *Derived Signals:* any function that relies on another signal.
//! 3. *Memos:* [`create_memo`], which returns a [`Memo`].
//! 4. *Resources:* [`create_resource`], which converts an `async` [`Future`](std::future::Future)
//!    into a synchronous [`Resource`] signal.
//! 5. *Triggers:* [`create_trigger`], creates a purely reactive [`Trigger`] primitive without any
//!    associated state.
//!
//! ### Effects
//! 1. Use [`create_effect`] when you need to synchronize the reactive system with something outside
//!    it (for example: logging to the console, writing to a file or local storage)
//! 2. The rooibos DOM renderer wraps any [`Fn`] in your template with [`create_effect`], so
//!    components you write do *not* need explicit effects to synchronize with the DOM.
//!
//! ### Example
//! ```
//! use rooibos_reactive::*;
//!
//! // creates a new reactive runtime
//! // this is omitted from most of the examples in the docs
//! // you usually won't need to call it yourself
//! let runtime = create_runtime();
//! // a signal: returns a (getter, setter) pair
//! let (count, set_count) = create_signal(0);
//!
//! // calling the getter gets the value
//! // can be `count()` on nightly
//! assert_eq!(count.get(), 0);
//! // calling the setter sets the value
//! // can be `set_count(1)` on nightly
//! set_count.set(1);
//! // or we can mutate it in place with update()
//! set_count.update(|n| *n += 1);
//!
//! // a derived signal: a plain closure that relies on the signal
//! // the closure will run whenever we *access* double_count()
//! let double_count = move || count.get() * 2;
//! assert_eq!(double_count(), 4);
//!
//! // a memo: subscribes to the signal
//! // the closure will run only when count changes
//! let memoized_triple_count = create_memo(move |_| count.get() * 3);
//! // can be `memoized_triple_count()` on nightly
//! assert_eq!(memoized_triple_count.get(), 6);
//!
//! // this effect will run whenever `count` changes
//! create_effect(move |_| {
//!     println!("Count = {}", count.get());
//! });
//!
//! // disposes of the reactive runtime
//! runtime.dispose();
//! ```

#[cfg_attr(debug_assertions, macro_use)]
extern crate tracing;

#[macro_use]
mod signal;
pub mod callback;
mod context;
#[macro_use]
mod diagnostics;
mod effect;
// contains "private" implementation details right now.
// could make this unhidden in the future if needed.
// macro_export makes it public from the crate root anyways
#[doc(hidden)]
pub mod macros;
mod memo;
mod node;
pub mod oco;
mod resource;
mod runtime;
mod selector;
#[cfg(any(doc, feature = "serde"))]
mod serde;
mod serialization;
mod signal_wrappers_read;
mod signal_wrappers_write;
mod slice;
mod spawn;
mod stored_value;
pub mod suspense;
mod trigger;
mod watch;

pub use callback::*;
pub use context::*;
pub use diagnostics::SpecialNonReactiveZone;
pub use effect::*;
pub use memo::*;
pub use node::Disposer;
pub use oco::*;
pub use resource::*;
use runtime::*;
pub use runtime::{
    as_child_of_current_owner, batch, create_runtime, current_runtime, on_cleanup, run_as_child,
    set_current_runtime, spawn_local_with_current_owner, spawn_local_with_owner, try_batch,
    try_spawn_local_with_current_owner, try_spawn_local_with_owner, try_with_owner, untrack,
    untrack_with_diagnostics, with_current_owner, with_owner, Owner, RuntimeId, ScopedFuture,
};
pub use selector::*;
pub use serialization::*;
pub use signal::{prelude as signal_prelude, *};
pub use signal_wrappers_read::*;
pub use signal_wrappers_write::*;
pub use slice::*;
pub use spawn::*;
pub use stored_value::*;
pub use suspense::{GlobalSuspenseContext, SuspenseContext};
pub use tracing::warn;
pub use trigger::*;
pub use watch::*;

#[doc(hidden)]
pub fn console_warn(s: &str) {
    eprintln!("{s}");
}
