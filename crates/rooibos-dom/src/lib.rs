mod dom;
mod events;
mod widgets;

// hack for resolving self in proc macros https://github.com/bkchr/proc-macro-crate/issues/14#issuecomment-1742071768
extern crate self as rooibos_dom;

pub use dom::*;
pub use events::*;
pub use typed_builder;
pub use widgets::*;

#[macro_export]
macro_rules! signal {
    ($($x:tt)*) => {
        Signal::derive(move || $($x)*)
    };
}
