mod dom;
mod widgets;

// hack for resolving self in proc macros https://github.com/bkchr/proc-macro-crate/issues/14#issuecomment-1742071768
extern crate self as rooibos_dom;

pub use dom::*;
pub use rooibos_dom_macros::*;
pub use typed_builder;
pub use widgets::*;
mod chart;
mod components;
mod sparkline;

pub mod prelude {
    pub use chart::*;
    pub use components::*;
    pub use ratatui::layout::*;
    pub use ratatui::style::*;
    pub use ratatui::text::*;
    pub use ratatui::widgets::*;
    pub use ratatui::{symbols, Frame};
    pub use sparkline::*;

    pub use super::*;
}
