mod dom;
mod events;
mod suspense;
mod widgets;

// hack for resolving self in proc macros https://github.com/bkchr/proc-macro-crate/issues/14#issuecomment-1742071768
extern crate self as rooibos_dom;

pub use dom::*;
pub use events::*;
pub use suspense::*;
pub use widgets::*;

#[macro_export]
macro_rules! signal {
    ($($x:tt)*) => {
        Signal::derive(move || $($x)*)
    };
}
