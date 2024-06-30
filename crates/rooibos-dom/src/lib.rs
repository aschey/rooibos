mod dom;
mod error_boundary;
mod events;
mod suspense;
mod widgets;

pub use dom::*;
pub use error_boundary::*;
pub use events::*;
#[doc(hidden)]
pub use ratatui::text as __text;
#[doc(hidden)]
pub use ratatui::widgets as __widgets;
#[doc(hidden)]
pub use reactive_graph as __reactive;
pub use suspense::*;
pub use tachys::reactive_graph as __tachys_reactive;
pub use terminput::*;
pub use throw_error::*;
pub use widgets::*;

#[macro_export]
macro_rules! derive_signal {
    ($($x:tt)*) => {
        $crate::__reactive::wrappers::read::Signal::derive(move || $($x)*)
    };
}

#[macro_export]
macro_rules! line {
    () => {
        $crate::__text::Line::default()
    };
    ($span:expr; $n:expr) => {
        $crate::__text::Line::from(vec![$span.into(); $n])
    };
    ($($span:expr),+ $(,)?) => {{
        $crate::__text::Line::from(vec![
        $(
            $span.into(),
        )+
        ])
    }};
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Focused,
    Active,
    Default,
}
