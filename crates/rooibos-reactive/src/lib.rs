//! Reactive primitives for [Sycamore](https://github.com/sycamore-rs/sycamore).
//!
//! ```rust
//! use sycamore_reactive3::*;
//!
//! create_root(|cx| {
//!     let greeting = create_signal(cx, "Hello");
//!     let name = create_signal(cx, "World");
//!
//!     let display_text = create_memo(cx, move || format!("{greeting} {name}!"));
//!     assert_eq!(display_text.get_clone(), "Hello World!");
//!
//!     name.set("Sycamore");
//!     assert_eq!(display_text.get_clone(), "Hello Sycamore!");
//! });
//! ```
//!
//! # A note on `nightly`
//!
//! If you are using rust `nightly`, you can enable the `nightly` feature to enable the more terse
//! syntax for signal get/set.
//!
//! ```rust
//! # use sycamore_reactive3::*;
//! # create_root(|cx| {
//! let signal = create_signal(cx, 123);
//!
//! // Stable:
//! let value = signal.get();
//! signal.set(456);
//!
//! // Nightly:
//! let value = signal();
//! signal(456);
//! # });
//! ```
//! Of course, the stable `.get()` also works on nightly as well if that's what you prefer.

mod context;
mod effects;
mod iter;
mod memos;
mod scope;
mod signals;
mod store;
mod stored_values;
mod utils;

pub use context::*;
pub use effects::*;
pub use iter::*;
pub use memos::*;
pub use rooibos_reactive_macros::*;
pub use scope::*;
pub use signals::*;
pub use store::*;
pub use stored_values::*;
pub use utils::*;
