mod dom;
mod error_boundary;
mod events;
mod suspense;
mod widgets;

pub use dom::*;
pub use error_boundary::*;
pub use events::*;
pub use suspense::*;
pub use throw_error::*;
pub use widgets::*;

#[macro_export]
macro_rules! signal {
    ($($x:tt)*) => {
        Signal::derive(move || $($x)*)
    };
}
