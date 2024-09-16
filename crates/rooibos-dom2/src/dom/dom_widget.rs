use std::any::type_name;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::widgets::WidgetRole;
use crate::{next_node_id, refresh_dom, Role};

pub(crate) type DomWidgetFn = Box<dyn FnMut(Rect, &mut Buffer)>;

#[derive(Clone)]
pub struct DomWidgetNode {
    f: Rc<dyn Fn() -> DomWidgetFn>,
    rc_f: Rc<RefCell<DomWidgetFn>>,
    id: u32,
    pub(crate) widget_type: String,
    pub(crate) role: Option<Role>,
}

impl PartialEq for DomWidgetNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomWidgetNode {}

impl Debug for DomWidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}/>", self.widget_type)
    }
}

impl DomWidgetNode {
    pub fn new<T: 'static, F1: Fn() -> F2 + 'static, F2: FnMut(Rect, &mut Buffer) + 'static>(
        f: F1,
    ) -> Self {
        let widget_type = type_name::<T>();
        let role = T::widget_role();
        let id = next_node_id();
        let rc_f: Rc<RefCell<DomWidgetFn>> = Rc::new(RefCell::new(Box::new(f())));

        Self {
            id,
            role,
            rc_f,
            f: Rc::new(move || Box::new((f)())),
            widget_type: widget_type.into(),
        }
    }

    pub(crate) fn render(&self, rect: Rect, buf: &mut Buffer) {
        (*self.rc_f).borrow_mut()(rect, buf);
    }

    pub fn build(&self) {
        (*self.rc_f.borrow_mut()) = (self.f)();
        refresh_dom();
    }
}
