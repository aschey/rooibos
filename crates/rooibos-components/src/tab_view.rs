use std::ops::{Deref, DerefMut};

use ratatui::layout::Constraint;
use ratatui::layout::Constraint::*;
use ratatui::style::{Style, Styled};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Tabs};
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeProp, MaybeSignal, Signal};
use rooibos_dom::{
    col, derive_signal, widget_ref, ChildrenFn, Constrainable, EventData, IntoAny, IntoChildrenFn,
    KeyEvent, MouseEvent, Render,
};

#[derive(Clone)]
pub struct Tab {
    header: MaybeSignal<Line<'static>>,
    decorator: Option<MaybeSignal<Line<'static>>>,
    value: String,
    children: ChildrenFn,
}

#[derive(Clone)]
pub struct TabList(pub Vec<Tab>);

impl Deref for TabList {
    type Target = Vec<Tab>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TabList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TabList {
    pub fn next_tab(&self, focused: &str) -> Option<&Tab> {
        if self.is_empty() {
            return None;
        }
        let current = self.iter().position(|t| t.value == focused).unwrap();
        if current == self.len() - 1 {
            self.first()
        } else {
            Some(&self[current + 1])
        }
    }

    pub fn prev_tab(&self, focused: &str) -> Option<&Tab> {
        if self.is_empty() {
            return None;
        }
        let current = self.iter().position(|t| t.value == focused).unwrap();
        if current == 0 {
            self.last()
        } else {
            Some(&self[current - 1])
        }
    }
}

impl Tab {
    pub fn new(
        header: impl Into<MaybeSignal<Line<'static>>>,
        value: impl Into<String>,
        children: impl IntoChildrenFn,
    ) -> Self {
        Self {
            header: header.into(),
            value: value.into(),
            decorator: Default::default(),
            children: children.into_children_fn(),
        }
    }

    pub fn decorator(mut self, decorator: impl Into<MaybeSignal<Line<'static>>>) -> Self {
        self.decorator = Some(decorator.into());
        self
    }

    pub fn get_header(&self) -> Line<'static> {
        self.header.get()
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

type OnChangeFn = dyn FnMut(usize, &str);

pub struct TabView {
    block: MaybeProp<Block<'static>>,
    highlight_style: MaybeSignal<Style>,
    decorator_highlight_style: MaybeSignal<Style>,
    style: MaybeSignal<Style>,
    on_title_click: Box<OnChangeFn>,
    on_decorator_click: Box<OnChangeFn>,
    on_focus: Box<dyn FnMut(EventData)>,
    on_blur: Box<dyn FnMut(EventData)>,
    on_key_down: Box<dyn FnMut(KeyEvent, EventData)>,
    constraint: MaybeSignal<Constraint>,
    fit: MaybeSignal<bool>,
    divider: MaybeSignal<Span<'static>>,
    header_constraint: MaybeSignal<Constraint>,
}

impl Default for TabView {
    fn default() -> Self {
        Self {
            on_title_click: Box::new(move |_, _| {}),
            on_decorator_click: Box::new(move |_, _| {}),
            on_key_down: Box::new(move |_, _| {}),
            on_focus: Box::new(move |_| {}),
            on_blur: Box::new(move |_| {}),
            block: Default::default(),
            highlight_style: Default::default(),
            decorator_highlight_style: Default::default(),
            style: Default::default(),
            constraint: Default::default(),
            fit: false.into(),
            divider: Span::raw(symbols::line::VERTICAL).into(),
            header_constraint: Constraint::Length(1).into(),
        }
    }
}

impl Constrainable for TabView {
    fn constraint<S>(mut self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        self.constraint = constraint.into();
        self
    }
}

impl TabView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block(mut self, block: impl Into<MaybeProp<Block<'static>>>) -> Self {
        self.block = block.into();
        self
    }

    pub fn header_constraint(
        mut self,
        header_constraint: impl Into<MaybeSignal<Constraint>>,
    ) -> Self {
        self.header_constraint = header_constraint.into();
        self
    }

    pub fn highlight_style(mut self, highlight_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.highlight_style = highlight_style.into();
        self
    }

    pub fn decorator_highlight_style(
        mut self,
        decorator_highlight_style: impl Into<MaybeSignal<Style>>,
    ) -> Self {
        self.decorator_highlight_style = decorator_highlight_style.into();
        self
    }

    pub fn style(mut self, style: impl Into<MaybeSignal<Style>>) -> Self {
        self.style = style.into();
        self
    }

    pub fn fit(mut self, fit: impl Into<MaybeSignal<bool>>) -> Self {
        self.fit = fit.into();
        self
    }

    pub fn divider(mut self, divider: impl Into<MaybeSignal<Span<'static>>>) -> Self {
        self.divider = divider.into();
        self
    }

    pub fn on_title_click(mut self, on_title_click: impl FnMut(usize, &str) + 'static) -> Self {
        self.on_title_click = Box::new(on_title_click);
        self
    }

    pub fn on_key_down(mut self, on_key_down: impl FnMut(KeyEvent, EventData) + 'static) -> Self {
        self.on_key_down = Box::new(on_key_down);
        self
    }

    pub fn on_focus(mut self, on_focus: impl FnMut(EventData) + 'static) -> Self {
        self.on_focus = Box::new(on_focus);
        self
    }

    pub fn on_blur(mut self, on_blur: impl FnMut(EventData) + 'static) -> Self {
        self.on_blur = Box::new(on_blur);
        self
    }

    pub fn on_decorator_click(
        mut self,
        on_decorator_click: impl FnMut(usize, &str) + 'static,
    ) -> Self {
        self.on_decorator_click = Box::new(on_decorator_click);
        self
    }

    pub fn render(
        self,
        current_tab: impl Into<Signal<String>>,
        children: impl Into<MaybeSignal<TabList>>,
    ) -> impl Render {
        let Self {
            block,
            highlight_style,
            decorator_highlight_style,
            style,
            mut on_title_click,
            mut on_decorator_click,
            on_focus,
            on_blur,
            on_key_down,
            constraint,
            fit,
            header_constraint,
            divider,
        } = self;

        let children: MaybeSignal<TabList> = children.into();
        let current_tab: Signal<String> = current_tab.into();

        let cur_tab = {
            let children = children.clone();
            derive_signal!({
                let current_tab = current_tab.get();
                children.get().iter().enumerate().find_map(|(i, c)| {
                    if c.value == current_tab {
                        Some((c.children.clone(), i))
                    } else {
                        None
                    }
                })
            })
        };

        let headers = {
            let children = children.clone();
            derive_signal!({
                let cur_tab = cur_tab.get();
                let Some((_, cur_tab)) = cur_tab else {
                    return vec![];
                };
                let highlight_style = highlight_style.get();
                let decorator_highlight_style = decorator_highlight_style.get();
                children
                    .get()
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let mut header = t.header.get();

                        if let Some(decorator) = &t.decorator {
                            let mut spans = header.spans;
                            let mut decorator_spans = decorator.get().spans;
                            if i == cur_tab {
                                spans = spans
                                    .into_iter()
                                    .map(|s| s.set_style(highlight_style))
                                    .collect();
                                decorator_spans = decorator_spans
                                    .into_iter()
                                    .map(|s| s.set_style(decorator_highlight_style))
                                    .collect();
                            }
                            Line::from([spans, vec![Span::from("  ")], decorator_spans].concat())
                        } else {
                            if i == cur_tab {
                                let spans: Vec<_> = header
                                    .spans
                                    .into_iter()
                                    .map(|s| s.set_style(highlight_style))
                                    .collect();
                                header = Line::from(spans);
                            }

                            header
                        }
                    })
                    .collect::<Vec<_>>()
            })
        };

        let divider_width = {
            let divider = divider.clone();
            derive_signal!(divider.get().width())
        };

        let headers_len = derive_signal!({
            let headers = headers.get();
            let divider_width = divider_width.get() as u16;
            let len = headers.len();
            if len == 0 {
                return 0;
            }
            // title length + 2 spaces per title + number of dividers (number of tabs - 1)
            // + outside borders (2)
            headers.iter().map(|t| (t.width() + 2) as u16).sum::<u16>()
                + ((len as u16 - 1) * divider_width)
                + 2
        });

        let constraint = derive_signal!({
            if fit.get() {
                Length(headers_len.get())
            } else {
                constraint.get()
            }
        });

        let on_click = move |mouse_event: MouseEvent, event_data: EventData| {
            let start_col = event_data.rect.x;
            let col_offset = mouse_event.column - start_col;
            let children = children.get();
            let divider_width = divider_width.get() as u16;
            let mut total_len = 1u16;
            for (i, child) in children.iter().enumerate() {
                let header = child.header.get();
                let decorator_area = child
                    .decorator
                    .as_ref()
                    .map(|d| (d.get().width() + 2) as u16)
                    .unwrap_or(0);
                let header_area = header.width() as u16 + 2;

                if col_offset <= (total_len + header_area) {
                    if child.value != current_tab.get() {
                        on_title_click(i, &child.value);
                    }
                    break;
                }
                if col_offset <= (total_len + header_area + decorator_area) {
                    on_decorator_click(i, &child.value);
                    break;
                }
                total_len += header_area + decorator_area + divider_width;
            }
        };

        col![
            widget_ref!({
                let headers = Tabs::new(headers.get())
                    .divider(divider.get())
                    .style(style.get())
                    .highlight_style(Style::default())
                    .select(cur_tab.get().map(|t| t.1).unwrap_or(0));
                if let Some(block) = block.get() {
                    headers.block(block)
                } else {
                    headers
                }
            })
            .focusable(true)
            .on_click(on_click)
            .on_key_down(on_key_down)
            .on_focus(on_focus)
            .on_blur(on_blur)
            .constraint(header_constraint.get()),
            move || cur_tab
                .get()
                .map(|c| c.0())
                .unwrap_or_else(|| ().into_any())
        ]
        .constraint(constraint)
    }
}
