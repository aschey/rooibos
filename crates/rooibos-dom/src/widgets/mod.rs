mod button;
mod chart;
mod sparkline;

use std::any::{Any, TypeId};

pub use button::*;
pub use chart::*;
use ratatui::widgets::{List, Paragraph, Tabs};
pub use sparkline::*;
use taffy::Size;

use crate::MeasureNode;

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

impl MeasureNode for Tabs<'_> {
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        Size::zero()
    }
}

impl MeasureNode for List<'_> {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> Size<f32> {
        Size::zero()
    }
}
