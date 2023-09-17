use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use prelude::*;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::{symbols, Frame};
use reactive::{create_child_scope, StoredValue};
pub use rooibos_macros::*;
use rooibos_reactive::Scope;
use typemap_ors::{Key, TypeMap};
pub use {rooibos_reactive as reactive, typed_builder};

pub mod prelude {
    pub use ratatui::layout::*;
    pub use ratatui::style::*;
    pub use ratatui::text::*;
    pub use ratatui::widgets::*;
    pub use ratatui::{symbols, Frame};

    pub use super::components::*;
    pub use super::*;
}
pub mod components;

pub trait Backend: ratatui::backend::Backend + 'static {}

impl<B> Backend for B where B: ratatui::backend::Backend + 'static {}

macro_rules! impl_widget {
    ($name:ident, $widget:ident, $props:ident) => {
        pub type $props<'a> = $widget<'a>;

        impl MakeBuilder for $props<'_> {}

        pub fn $name<B: Backend>(_cx: Scope, props: $props<'static>) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(props.clone(), rect)
        }
    };
}

macro_rules! impl_widget_no_lifetime {
    ($name:ident, $widget:ident, $props:ident) => {
        pub type $props = $widget;

        impl MakeBuilder for $props {}

        pub fn $name<B: Backend>(_cx: Scope, props: $props) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(props.clone(), rect)
        }
    };
}

macro_rules! impl_stateful_widget {
    ($name:ident, $widget:ident, $props:ident, $state:ident) => {
        impl<'a, B> StatefulRender<B, $props<'a>> for RefCell<$state>
        where
            B: Backend,
        {
            fn render_with_state(&mut self, widget: $props, frame: &mut Frame<B>, rect: Rect) {
                frame.render_stateful_widget(widget, rect, &mut self.borrow_mut())
            }
        }

        impl<'a, B> StatefulRender<B, $props<'a>> for $state
        where
            B: Backend,
        {
            fn render_with_state(&mut self, widget: $props, frame: &mut Frame<B>, rect: Rect) {
                frame.render_stateful_widget(widget, rect, &mut self.clone())
            }
        }

        pub type $props<'a> = $widget<'a>;

        pub fn $name<'a, B: Backend>(
            _cx: Scope,
            props: $props<'static>,
            mut state: impl StatefulRender<B, $widget<'a>> + 'static,
        ) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| {
                state.render_with_state(props.clone(), frame, rect);
            }
        }
    };
}

pub trait StatefulRender<B, W>
where
    B: Backend,
    W: StatefulWidget,
{
    fn render_with_state(&mut self, widget: W, frame: &mut Frame<B>, rect: Rect);
}

pub struct KeyData<B: Backend> {
    pub cx: Scope,
    pub stored_view: StoredValue<Rc<RefCell<dyn View<B>>>>,
    pub iteration: u32,
}

pub struct KeyWrapper<T>(PhantomData<T>);

impl<B: Backend> Key for KeyWrapper<B> {
    type Value = HashMap<(u64, u64), KeyData<B>>;
}

pub trait BuilderFacade {
    fn builder() -> Self;
}

pub trait BuildFacade {
    fn build(self) -> Self;
    fn __caller_id(self, caller_id: u64) -> Self;
}

pub trait MakeBuilder {}

impl<T> BuilderFacade for T
where
    T: MakeBuilder + Default,
{
    fn builder() -> Self {
        Self::default()
    }
}

impl<T> BuildFacade for T
where
    T: MakeBuilder,
{
    fn build(self) -> Self {
        self
    }

    fn __caller_id(self, _caller_id: u64) -> Self {
        self
    }
}

impl<'a> MakeBuilder for Row<'a> {}
impl<'a> MakeBuilder for Cell<'a> {}
impl<'a> MakeBuilder for Span<'a> {}
impl<'a> MakeBuilder for ListItem<'a> {}
impl<'a> MakeBuilder for Line<'a> {}
impl<'a> MakeBuilder for Text<'a> {}
impl MakeBuilder for Style {}
impl MakeBuilder for ListState {}
impl MakeBuilder for TableState {}
impl MakeBuilder for Wrap {}

impl_widget!(block, Block, BlockProps);
impl_widget!(paragraph, Paragraph, ParagraphProps);
impl_widget!(list, List, ListProps);
impl_widget!(tabs, Tabs, TabsProps);
impl_widget!(table, Table, TableProps);
impl_widget!(gauge, Gauge, GaugeProps);
impl_widget!(line_gauge, LineGauge, LineGaugeProps);
impl_widget!(bar_chart, BarChart, BarChartProps);
impl_widget_no_lifetime!(clear, Clear, ClearProps);
impl_stateful_widget!(stateful_list, List, StatefulListProps, ListState);
impl_stateful_widget!(stateful_table, Table, StatefulTableProps, TableState);
impl_stateful_widget!(
    stateful_scrollbar,
    Scrollbar,
    StatefulScrollbarProps,
    ScrollbarState
);

#[derive(Clone, Default)]
pub struct SparklineProps<'a> {
    inner: Sparkline<'a>,
    data: Vec<u64>,
}

impl<'a> SparklineProps<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn data(mut self, data: Vec<u64>) -> Self {
        self.data = data;
        self
    }

    pub fn max(mut self, max: u64) -> Self {
        self.inner = self.inner.max(max);
        self
    }

    pub fn bar_set(mut self, bar_set: symbols::bar::Set) -> Self {
        self.inner = self.inner.bar_set(bar_set);
        self
    }

    pub fn direction(mut self, direction: RenderDirection) -> Self {
        self.inner = self.inner.direction(direction);
        self
    }
}

impl<'a> Widget for SparklineProps<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        self.inner.data(&self.data).render(area, buf)
    }
}

impl<'a> Styled for SparklineProps<'a> {
    type Item = SparklineProps<'a>;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}

impl MakeBuilder for SparklineProps<'_> {}

pub fn sparkline<B: Backend>(_cx: Scope, props: SparklineProps<'static>) -> impl View<B> {
    move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(props.clone(), rect)
}

pub trait View<B: Backend>: 'static {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
    fn into_boxed_view(self) -> Box<dyn View<B>>;
}

impl<B, F> View<B> for F
where
    B: Backend,
    F: FnMut(&mut Frame<B>, Rect) + 'static,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B> View<B> for Box<dyn View<B>>
where
    B: Backend,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (**self).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        self
    }
}

impl<B> View<B> for Vec<Box<dyn View<B>>>
where
    B: Backend,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        let size = rect.height / self.len() as u16;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(size); self.len()])
            .split(rect);
        for (i, item) in self.iter_mut().enumerate() {
            item.view(frame, chunks[i]);
        }
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B> View<B> for Rc<RefCell<dyn View<B>>>
where
    B: Backend,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B, V> View<B> for Rc<RefCell<V>>
where
    B: Backend,
    V: View<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

pub trait LazyView<B: Backend>: 'static {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
}

impl<B, F, Ret> LazyView<B> for F
where
    B: Backend,
    F: FnMut() -> Ret + 'static,
    Ret: View<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)().view(frame, rect)
    }
}

pub struct LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    f: F,
    _phantom: PhantomData<B>,
}

impl<B, F> LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<B, F> View<B> for LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self.f).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

pub trait NewExt<'a, T>
where
    Self: 'a,
{
    fn new(source: T) -> Self;
}

pub trait NewFrom {}

impl<'a, S, T> NewExt<'a, T> for S
where
    S: NewFrom + 'a,
    Self: From<T>,
{
    fn new(source: T) -> Self {
        Self::from(source)
    }
}

impl<'a> NewFrom for Line<'a> {}
impl<'a> NewFrom for Span<'a> {}
impl<'a> NewFrom for Cell<'a> {}
impl<'a> NewFrom for Text<'a> {}

pub trait StyleExt<'a> {
    fn style(self, style: Style) -> Self;
}

impl<'a> StyleExt<'a> for Span<'a> {
    fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> StyleExt<'a> for Text<'a> {
    fn style(mut self, style: Style) -> Self {
        self.reset_style();
        self.patch_style(style);
        self
    }
}

pub trait IntoBoxed<T: ?Sized> {
    fn into_boxed(self) -> Box<T>;
}

impl<F, R> IntoBoxed<dyn Fn() -> R> for F
where
    F: Fn() -> R + 'static,
{
    fn into_boxed(self: F) -> Box<dyn Fn() -> R> {
        Box::new(self)
    }
}

pub trait IntoBoxedLazyView<B>
where
    B: Backend,
{
    fn into_boxed_lazy_view(self) -> Box<dyn Fn() -> Box<dyn View<B>>>;
}

impl<B, F, V> IntoBoxedLazyView<B> for F
where
    B: Backend,
    F: Fn() -> V + 'static,
    V: View<B>,
{
    fn into_boxed_lazy_view(self) -> Box<dyn Fn() -> Box<dyn View<B>>> {
        (move || (self)().into_boxed_view()).into_boxed()
    }
}

pub trait WrapExt {
    fn trim(self, trim: bool) -> Self;
}

impl WrapExt for Wrap {
    fn trim(self, trim: bool) -> Self {
        Self { trim }
    }
}

pub struct WidgetCache {
    pub view_cache: RefCell<TypeMap>,
    pub scope_cache: ScopeCache,
    iteration: AtomicU32,
}

impl WidgetCache {
    pub fn next_iteration(&self) {
        self.iteration.fetch_add(1, Ordering::SeqCst);
    }

    pub fn mark<B: Backend>(&self, node: &mut KeyData<B>) {
        let iter = self.iteration.load(Ordering::SeqCst);
        node.iteration = iter;
    }

    pub fn evict<B: Backend>(&self) {
        let mut cache_mut = self.view_cache.borrow_mut();
        let current_iteration = self.iteration.load(Ordering::SeqCst);

        if let Some(wrapper) = cache_mut.get_mut::<KeyWrapper<B>>() {
            for val in wrapper.values() {
                if val.iteration < current_iteration && !val.cx.is_disposed() && !val.cx.is_root() {
                    val.cx.dispose();
                }
            }

            let keys: Vec<_> = wrapper.keys().copied().collect();
            for k in &keys {
                if let Some(val) = wrapper.get(k) {
                    if val.cx.is_disposed() {
                        wrapper.remove(k);
                    }
                }
            }

            self.scope_cache.evict();
        }
    }
}

#[derive(Default)]
pub struct ScopeCache {
    scopes: Rc<RefCell<HashMap<(u64, u64, Option<u64>), Scope>>>,
}

impl ScopeCache {
    pub fn get_or_create(&self, cx: Scope, caller_id: u64, key: Option<u64>) -> Scope {
        let mut scopes = self.scopes.borrow_mut();
        if let Some(child_cx) = scopes.get(&(cx.id(), caller_id, key)) {
            *child_cx
        } else {
            let child_cx = create_child_scope(cx);
            scopes.insert((cx.id(), caller_id, key), child_cx);
            child_cx
        }
    }

    fn evict(&self) {
        let mut scopes = self.scopes.borrow_mut();
        let keys: Vec<_> = scopes.keys().copied().collect();
        for k in keys {
            if let Some(val) = scopes.get(&k) {
                if val.is_disposed() {
                    scopes.remove(&k);
                }
            }
        }
    }
}

thread_local! {
    pub static WIDGET_CACHE: WidgetCache = WidgetCache {
        view_cache: RefCell::new(TypeMap::new()),
        scope_cache: ScopeCache::default(),
        iteration: AtomicU32::new(0)
    };

}
