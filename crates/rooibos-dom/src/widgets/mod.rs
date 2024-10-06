mod button;
mod chart;
mod sparkline;

use std::any::{Any, TypeId};

pub use button::*;
pub use chart::*;
pub use sparkline::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Button,
    TextInput,
}

pub trait WidgetRole {
    fn widget_role() -> Option<Role>;
}

impl<T> WidgetRole for T
where
    T: Any,
{
    fn widget_role() -> Option<Role> {
        let current_type = TypeId::of::<Self>();

        if current_type == TypeId::of::<Button>() {
            return Some(Role::Button);
        }
        #[cfg(feature = "input")]
        if current_type == TypeId::of::<tui_textarea::TextArea>() {
            return Some(Role::TextInput);
        }
        None
    }
}
