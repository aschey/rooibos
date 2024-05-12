use ratatui::layout::Constraint;
use ratatui::layout::Constraint::*;
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Tabs};
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeProp, MaybeSignal, Signal};
use rooibos_dom::{
    col, signal, widget_ref, ChildrenFn, Constrainable, EventData, IntoChildrenFn, MouseEvent,
    Render,
};

#[derive(Clone)]
pub struct Tab {
    header: MaybeSignal<Line<'static>>,
    decorator: Option<MaybeSignal<Line<'static>>>,
    value: String,
    children: ChildrenFn,
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
    padding: MaybeProp<u16>,
    highlight_style: MaybeSignal<Style>,
    on_change: Box<OnChangeFn>,
    on_decorator_click: Box<OnChangeFn>,
    constraint: MaybeSignal<Constraint>,
    fit: MaybeSignal<bool>,
}

impl Default for TabView {
    fn default() -> Self {
        Self {
            on_change: Box::new(move |_, _| {}),
            on_decorator_click: Box::new(move |_, _| {}),
            padding: Default::default(),
            block: Default::default(),
            highlight_style: Default::default(),
            constraint: Default::default(),
            fit: false.into(),
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

    pub fn padding(mut self, padding: impl Into<MaybeProp<u16>>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn highlight_style(mut self, highlight_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.highlight_style = highlight_style.into();
        self
    }

    pub fn fit(mut self, fit: impl Into<MaybeSignal<bool>>) -> Self {
        self.fit = fit.into();
        self
    }

    pub fn on_change(mut self, on_change: impl FnMut(usize, &str) + 'static) -> Self {
        self.on_change = Box::new(on_change);
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
        children: impl Into<MaybeSignal<Vec<Tab>>>,
    ) -> impl Render {
        let Self {
            block,
            padding,
            highlight_style,
            mut on_change,
            mut on_decorator_click,
            constraint,
            fit,
        } = self;
        let children: MaybeSignal<Vec<Tab>> = children.into();

        let current_tab: Signal<String> = current_tab.into();

        let cur_tab = {
            let children = children.clone();
            signal!({
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
            signal!({
                let cur_tab = cur_tab.get().unwrap().1;
                let highlight_style = highlight_style.get();
                children
                    .get()
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let mut header = t.header.get();

                        if let Some(decorator) = &t.decorator {
                            let mut spans = header.spans;
                            if i == cur_tab {
                                spans = spans
                                    .into_iter()
                                    .map(|s| s.set_style(highlight_style))
                                    .collect();
                            }
                            Line::from(
                                [spans, vec![Span::from("  ")], decorator.get().spans].concat(),
                            )
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

        let headers_len = signal!({
            let headers = headers.get();
            let len = headers.len();
            // title length + 2 spaces per title + number of dividers (number of tabs - 1)
            // + outside borders (2)
            headers.iter().map(|t| (t.width() + 2) as u16).sum::<u16>() + (len as u16 - 1) + 2
        });

        let constraint = signal!({
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
            let mut total_len = 1u16;
            for (i, child) in children.iter().enumerate() {
                let header = child.header.get();
                let decorator = child
                    .decorator
                    .as_ref()
                    .map(|d| d.get())
                    .unwrap_or_default();
                let header_area = header.width() as u16 + 2;
                let decorator_area = decorator.width() as u16 + 2;
                if col_offset <= (total_len + header_area) {
                    if child.value != current_tab.get() {
                        on_change(i, &child.value);
                    }

                    break;
                }
                if col_offset <= (total_len + header_area + decorator_area) {
                    on_decorator_click(i, &child.value);

                    break;
                }
                total_len += header_area + decorator_area + 1;
            }
        };

        let length = signal!(padding.get().unwrap_or(0) * 2 + 1);

        col![
            widget_ref!({
                let headers = Tabs::new(headers.get())
                    .highlight_style(Style::default())
                    .select(cur_tab.get().unwrap().1);
                if let Some(block) = block.get() {
                    headers.block(block)
                } else {
                    headers
                }
            })
            .on_click(on_click)
            .length(length),
            move || cur_tab.get().unwrap().0()
        ]
        .constraint(constraint)
    }
}
