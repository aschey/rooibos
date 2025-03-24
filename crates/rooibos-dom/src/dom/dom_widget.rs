use std::any::type_name;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Context};
use ratatui::widgets::{BarChart, Gauge, LineGauge, Table};
use taffy::{AvailableSpace, Size, Style};
use unicode_width::UnicodeWidthStr;
use wasm_compat::cell::BoolCell;

use super::{DomNodeKey, with_nodes_mut};
use crate::widgets::{Role, WidgetRole};
use crate::{next_node_id, refresh_dom};

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
    fn render(&mut self, _rect: Rect, _frame: &mut Frame) {}
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

    fn estimate_size(&self) -> Size<f32>;
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

    fn estimate_size(&self) -> Size<f32> {
        *self
    }
}

impl MeasureNode for () {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
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

    fn estimate_size(&self) -> Size<f32> {
        (**self).estimate_size()
    }
}

impl MeasureNode for String {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width_cjk() as f32,
            height: 1.0,
        }
    }

    fn estimate_size(&self) -> Size<f32> {
        Size {
            width: self.width_cjk() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for &str {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width_cjk() as f32,
            height: 1.0,
        }
    }

    fn estimate_size(&self) -> Size<f32> {
        Size {
            width: self.width_cjk() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for Span<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: 1.0,
        }
    }

    fn estimate_size(&self) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for Line<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: 1.0,
        }
    }

    fn estimate_size(&self) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: 1.0,
        }
    }
}

impl MeasureNode for Text<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        // TODO: figure out wrapping
        // if let Some(width) = available_space.width.into_option() {
        //     let wrapped = wrap_text(
        //         self,
        //         wrap::Options {
        //             width: width as usize,
        //             initial_indent: Span::default(),
        //             subsequent_indent: Span::default(),
        //             break_words: true,
        //         },
        //     );
        //     return Size {
        //         width: wrapped.width() as f32,
        //         height: wrapped.height() as f32,
        //     };
        // }
        Size {
            width: self.width() as f32,
            height: self.height() as f32,
        }
    }

    fn estimate_size(&self) -> Size<f32> {
        Size {
            width: self.width() as f32,
            height: self.height() as f32,
        }
    }
}

impl MeasureNode for Gauge<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl MeasureNode for LineGauge<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl MeasureNode for Table<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl MeasureNode for BarChart<'_> {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

impl<F> MeasureNode for Canvas<'_, F>
where
    F: Fn(&mut Context),
{
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &Style,
    ) -> Size<f32> {
        Size::zero()
    }

    fn estimate_size(&self) -> Size<f32> {
        Size::zero()
    }
}

#[derive(Clone)]
pub struct DomWidgetNode {
    build_render_node: Rc<dyn Fn() -> Box<dyn RenderMeasure>>,
    widget_fn: Rc<RefCell<Box<dyn RenderMeasure>>>,
    id: u32,
    recompute_pending: Rc<BoolCell>,
    current_size: Size<f32>,
    pub(crate) widget_type: String,
    pub(crate) role: Option<Role>,
    key: DomNodeKey,
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

        Self {
            id,
            role,
            build_render_node: Rc::new(move || Box::new(render_node.build_renderer())),
            widget_fn: Rc::new(RefCell::new(Box::new(()))),
            recompute_pending: Rc::new(BoolCell::new(false)),
            current_size: Size::zero(),
            widget_type: widget_type.into(),
            key: DomNodeKey::default(),
        }
    }

    pub(crate) fn set_key(&mut self, key: DomNodeKey) {
        self.key = key;
    }

    pub(crate) fn render(&self, rect: Rect, frame: &mut Frame) {
        (*self.widget_fn).borrow_mut().render(rect, frame);
    }

    pub fn build(&self) {
        let props = (self.build_render_node)();
        *self.widget_fn.borrow_mut() = props;
        refresh_dom();
    }

    pub fn estimate_size(&self) {
        if self.recompute_pending.get() {
            return;
        }
        let size = self.widget_fn.borrow().estimate_size();
        if size != self.current_size {
            with_nodes_mut(|n| {
                n.force_recompute_layout(self.key);
            });
            self.recompute_pending.set(true);
        }
    }

    pub(crate) fn recompute_done(&self) {
        self.recompute_pending.set(false);
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

    fn estimate_size(&self) -> Size<f32> {
        self.widget_fn.borrow().estimate_size()
    }
}
