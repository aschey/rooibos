mod dom;
mod error_boundary;
mod events;
mod suspense;
mod widgets;

pub use dom::*;
pub use error_boundary::*;
pub use events::*;
pub use reactive_graph as __reactive;
pub use suspense::*;
pub use throw_error::*;
pub use widgets::*;

#[macro_export]
macro_rules! derive_signal {
    ($($x:tt)*) => {
        $crate::__reactive::wrappers::read::Signal::derive(move || $($x)*)
    };
}
