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
    ($string:literal) => {
        $crate::__text::Line::from(format!($string))
    };
    ($span:expr) => {
        $crate::__text::Line::from($span)
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

#[macro_export]
macro_rules! text {
    () => {
        $crate::__text::Text::default()
    };
    ($string:literal) => {
        $crate::__text::Text::from(format!($string))
    };
    ($line:expr; $n:expr) => {
        $crate::__text::::Text::from(vec![$line.into(); $n])
    };
    ($($line:expr),+ $(,)?) => {{
        $crate::__text::Text::from(vec![
        $(
            $line.into(),
        )+
        ])
    }};
}

#[macro_export]
macro_rules! span {
    ($string:literal) => {
        $crate::__text::Span::raw(format!($string))
    };
    ($expr:expr) => {
        $crate::__text::Span::raw($expr.to_string())
    };
    ($string:literal, $($arg:tt)*) => {
        $crate::__text::Span::raw(format!($string, $($arg)*))
    };
    ($style:expr, $($arg:tt)*) => {
        compile_error!("first parameter must be a formatting specifier followed by a comma OR a `Style` followed by a semicolon")
    };
    ($style:expr; $($arg:tt)*) => {
        $crate::__text::Span::styled(format!($($arg)*), $style)
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Focused,
    Active,
    Default,
}
