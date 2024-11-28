use std::any::type_name;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use taffy::{AvailableSpace, Size, Style};
use unicode_width::UnicodeWidthStr;

use crate::widgets::{Role, WidgetRole};
use crate::{next_node_id, refresh_dom};

pub(crate) type DomWidgetFn = Box<dyn FnMut(Rect, &mut Frame)>;

pub trait BuildNodeRenderer {
    fn build_renderer(&self) -> impl RenderNode + MeasureNode + 'static;
}

impl<F, RN> BuildNodeRenderer for F
where
    F: Fn() -> RN,
    RN: RenderNode + MeasureNode + 'static,
{
    fn build_renderer(&self) -> impl RenderNode + MeasureNode + 'static {
        self()
    }
}

pub trait RenderNode {
    fn render(&mut self, rect: Rect, frame: &mut Frame);
}

impl RenderNode for () {
    fn render(&mut self, rect: Rect, frame: &mut Frame) {}
}

impl RenderNode for Box<dyn RenderNode> {
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        (**self).render(rect, frame)
    }
}

trait RenderMeasure: RenderNode + MeasureNode {}

impl<T> RenderMeasure for T where T: RenderNode + MeasureNode {}

pub trait MeasureNode {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32>;
}

impl MeasureNode for Size<f32> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        *self
    }
}

impl MeasureNode for () {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }
}

impl MeasureNode for Box<dyn MeasureNode> {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        (**self).measure(known_dimensions, available_space, style)
    }
}

impl MeasureNode for String {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width_cjk() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for &str {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width_cjk() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for Span<'_> {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for Line<'_> {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for Text<'_> {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: self.height() as f32,
        }
    }
}

#[derive(Clone)]
pub struct DomWidgetNode {
    build_render_node: Rc<dyn Fn() -> Box<dyn RenderMeasure>>,
    //render_node: Rc<dyn RenderNodeBoxed>,
    widget_fn: Rc<RefCell<Box<dyn RenderMeasure>>>,
    //measure_fn: Rc<RefCell<Box<dyn MeasureNode>>>,
    //render_node: Rc<dyn BuildNodeRenderer>,
    //measure_node: Rc<dyn MeasureNode>,
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
    pub fn new<T, R>(render_node: R) -> Self
    where
        T: 'static,
        R: BuildNodeRenderer + 'static,
    {
        let widget_type = type_name::<T>();
        let role = T::widget_role();
        let id = next_node_id();
        //let rc_f: Rc<RefCell<DomWidgetFn>> = Rc::new(RefCell::new(Box::new(|_, _| {})));

        Self {
            id,
            role,
            build_render_node: Rc::new(move || Box::new(render_node.build_renderer())),
            widget_fn: Rc::new(RefCell::new(Box::new(()))),
            //measure_fn: Rc::new(RefCell::new(Box::new(Size::zero()))),
            //measure_node: Rc::new(()),
            // render_node: Rc::new(()),
            //rc_f,
            //measure_fn: Rc::new(move || render_node.get_measure_fn(props))
            //f: Rc::new(move || Box::new(render_node.get_render_fn(render_node.build_props()))),
            widget_type: widget_type.into(),
        }
    }

    pub(crate) fn render(&self, rect: Rect, frame: &mut Frame) {
        (*self.widget_fn).borrow_mut().render(rect, frame);
    }

    pub fn build(&self) {
        let props = (self.build_render_node)();
        *self.widget_fn.borrow_mut() = props;
        refresh_dom();
    }
}

impl MeasureNode for DomWidgetNode {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
    ) -> Size<f32> {
        self.widget_fn
            .borrow()
            .measure(known_dimensions, available_space, style)
    }
}
