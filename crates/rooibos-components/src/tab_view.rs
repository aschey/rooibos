use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Tabs};
use rooibos_dom::events::{
    BlurEvent, ClickEventProps, EventData, EventHandle, FocusEvent, KeyHandler,
};
use rooibos_dom::{IntoSpan, line};
use rooibos_reactive::any_view::IntoAny as _;
use rooibos_reactive::dom::layout::{Dimension, IntoDimensionSignal, full, height, max_height};
use rooibos_reactive::dom::{ChildrenFn, IntoChildrenFn, Render};
use rooibos_reactive::graph::IntoReactiveValue;
use rooibos_reactive::graph::traits::{Get, With};
use rooibos_reactive::graph::wrappers::read::{MaybeProp, Signal};
use rooibos_reactive::{col, wgt};
use rooibos_theme::{Style, Styled};

use crate::Keyed;
use crate::wrapping_list::KeyedWrappingList;

pub type TabList = KeyedWrappingList<Tab>;

#[derive(Clone)]
pub struct Tab {
    header: Signal<Line<'static>>,
    decorator: Option<Signal<Line<'static>>>,
    value: String,
    children: ChildrenFn,
}

impl Tab {
    pub fn new<M>(
        header: impl IntoReactiveValue<Signal<Line<'static>>, M>,
        value: impl Into<String>,
        children: impl IntoChildrenFn,
    ) -> Self {
        Self {
            header: header.into_reactive_value(),
            value: value.into(),
            decorator: Default::default(),
            children: children.into_children_fn(),
        }
    }

    pub fn decorator<M>(
        mut self,
        decorator: impl IntoReactiveValue<Signal<Line<'static>>, M>,
    ) -> Self {
        self.decorator = Some(decorator.into_reactive_value());
        self
    }

    pub fn get_header(&self) -> Line<'static> {
        self.header.get()
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

impl Keyed for Tab {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.value
    }
}

type OnChangeFn = dyn FnMut(usize, &str);

pub struct TabView {
    block: MaybeProp<Block<'static>>,
    highlight_style: Signal<Style>,
    decorator_highlight_style: Option<Signal<Style>>,
    style: Signal<Style>,
    on_title_click: Box<OnChangeFn>,
    on_decorator_click: Option<Box<OnChangeFn>>,
    on_direct_focus: Box<dyn FnMut(FocusEvent, EventData, EventHandle)>,
    on_direct_blur: Box<dyn FnMut(BlurEvent, EventData, EventHandle)>,
    on_key_down: Box<dyn KeyHandler>,
    fit: Signal<bool>,
    divider: Signal<Span<'static>>,
    header_height: Signal<Dimension>,
    width: Signal<Dimension>,
    padding_left: Signal<Line<'static>>,
    padding_right: Signal<Line<'static>>,
    body_height: Signal<Dimension>,
}

enum ClickAction<'a> {
    Decorator(usize, String, &'a mut Box<OnChangeFn>),
    Title(usize, String),
}

impl Default for TabView {
    fn default() -> Self {
        Self {
            on_title_click: Box::new(move |_, _| {}),
            on_decorator_click: None,
            on_key_down: Box::new(move |_| {}),
            on_direct_focus: Box::new(move |_, _, _| {}),
            on_direct_blur: Box::new(move |_, _, _| {}),
            block: Default::default(),
            highlight_style: Default::default(),
            decorator_highlight_style: Default::default(),
            style: Default::default(),
            fit: false.into(),
            divider: symbols::line::VERTICAL.into_span().into(),
            header_height: 1.into_dimension_signal(),
            width: full().into(),
            padding_left: line!(" ").into(),
            padding_right: line!(" ").into(),
            body_height: Dimension::Auto.into(),
        }
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

    pub fn header_height(mut self, header_height: impl IntoDimensionSignal) -> Self {
        self.header_height = header_height.into_dimension_signal();
        self
    }

    pub fn body_height(mut self, body_height: impl IntoDimensionSignal) -> Self {
        self.body_height = body_height.into_dimension_signal();
        self
    }

    pub fn width(mut self, width: impl IntoDimensionSignal) -> Self {
        self.width = width.into_dimension_signal();
        self
    }

    pub fn highlight_style<M>(
        mut self,
        highlight_style: impl IntoReactiveValue<Signal<Style>, M>,
    ) -> Self {
        self.highlight_style = highlight_style.into_reactive_value();
        self
    }

    pub fn decorator_highlight_style<M>(
        mut self,
        decorator_highlight_style: impl IntoReactiveValue<Signal<Style>, M>,
    ) -> Self {
        self.decorator_highlight_style = Some(decorator_highlight_style.into_reactive_value());
        self
    }

    pub fn style<M>(mut self, style: impl IntoReactiveValue<Signal<Style>, M>) -> Self {
        self.style = style.into_reactive_value();
        self
    }

    pub fn fit<M>(mut self, fit: impl IntoReactiveValue<Signal<bool>, M>) -> Self {
        self.fit = fit.into_reactive_value();
        self
    }

    pub fn divider<M>(mut self, divider: impl IntoReactiveValue<Signal<Span<'static>>, M>) -> Self {
        self.divider = divider.into_reactive_value();
        self
    }

    pub fn on_title_click(mut self, on_title_click: impl FnMut(usize, &str) + 'static) -> Self {
        self.on_title_click = Box::new(on_title_click);
        self
    }

    pub fn on_key_down(mut self, on_key_down: impl KeyHandler + 'static) -> Self {
        self.on_key_down = Box::new(on_key_down);
        self
    }

    pub fn on_direct_focus(
        mut self,
        on_direct_focus: impl FnMut(FocusEvent, EventData, EventHandle) + 'static,
    ) -> Self {
        self.on_direct_focus = Box::new(on_direct_focus);
        self
    }

    pub fn on_direct_blur(
        mut self,
        on_direct_blur: impl FnMut(BlurEvent, EventData, EventHandle) + 'static,
    ) -> Self {
        self.on_direct_blur = Box::new(on_direct_blur);
        self
    }

    pub fn on_decorator_click(
        mut self,
        on_decorator_click: impl FnMut(usize, &str) + 'static,
    ) -> Self {
        self.on_decorator_click = Some(Box::new(on_decorator_click));
        self
    }

    pub fn padding_left<M>(
        mut self,
        padding_left: impl IntoReactiveValue<Signal<Line<'static>>, M>,
    ) -> Self {
        self.padding_left = padding_left.into_reactive_value();
        self
    }

    pub fn padding_right<M>(
        mut self,
        padding_right: impl IntoReactiveValue<Signal<Line<'static>>, M>,
    ) -> Self {
        self.padding_right = padding_right.into_reactive_value();
        self
    }

    pub fn render<M1, M2>(
        self,
        current_tab: impl IntoReactiveValue<Signal<String>, M1>,
        children: impl IntoReactiveValue<Signal<TabList>, M2>,
    ) -> impl Render {
        let Self {
            block,
            highlight_style,
            decorator_highlight_style,
            style,
            mut on_title_click,
            mut on_decorator_click,
            on_direct_focus,
            on_direct_blur,
            on_key_down,
            width,
            fit,
            header_height,
            divider,
            padding_left,
            padding_right,
            body_height,
        } = self;

        let children: Signal<TabList> = children.into_reactive_value();
        let current_tab: Signal<String> = current_tab.into_reactive_value();

        let cur_tab = move || {
            current_tab.with(|current_tab| {
                children.with(|c| {
                    c.iter().enumerate().find_map(|(i, c)| {
                        if &c.value == current_tab {
                            Some((c.children.clone(), i))
                        } else {
                            None
                        }
                    })
                })
            })
        };

        let headers = move || {
            let cur_tab = cur_tab();
            let Some((_, cur_tab)) = cur_tab else {
                return vec![];
            };
            let highlight_style = highlight_style.get();
            let decorator_highlight_style = decorator_highlight_style.map(|s| s.get());
            children.with(|c| {
                c.iter()
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
                                if let Some(decorator_highlight_style) = decorator_highlight_style {
                                    decorator_spans = decorator_spans
                                        .into_iter()
                                        .map(|s| s.set_style(decorator_highlight_style))
                                        .collect();
                                }
                            }
                            line!([spans, vec!["  ".into()], decorator_spans].concat())
                        } else {
                            if i == cur_tab {
                                let spans: Vec<_> = header
                                    .spans
                                    .into_iter()
                                    .map(|s| s.set_style(highlight_style))
                                    .collect();
                                header = line!(spans);
                            }

                            header
                        }
                    })
                    .collect::<Vec<_>>()
            })
        };
        let divider_width = move || divider.with(|d| d.width());

        let padding_width_left = move || padding_left.with(|p| p.width());

        let padding_width_right = move || padding_right.with(|p| p.width());

        let headers_len = move || {
            let headers = headers();
            let headers_len = headers.len();
            let divider_width = divider_width() as u16;

            if headers_len == 0 {
                return 0;
            }
            let padding_width = padding_width_left() + padding_width_right();
            // title length + padding length + number of dividers (number of tabs - 1)
            // + outside borders (2)
            headers
                .iter()
                .map(|t| (t.width() + padding_width) as u16)
                .sum::<u16>()
                + ((headers_len as u16 - 1) * divider_width)
                + 2
        };

        let width = move || {
            if fit.get() {
                (headers_len() as u32).into()
            } else {
                width.get()
            }
        };

        let on_click = move |props: ClickEventProps| {
            let start_col = props.data.rect.x;
            let col_offset = props.event.column - start_col;

            let divider_width = divider_width() as u16;
            let mut total_len = 1u16;
            let current_tab = current_tab.get();
            let padding_width_left = padding_width_left() as u16;
            let padding_width_right = padding_width_right() as u16;
            // Event handlers could access the children object so we shouldn't invoke them until
            // we're out of the with() block
            let action = children.with(|c| {
                for (i, child) in c.iter().enumerate() {
                    let header_width = child.header.with(|h| h.width() as u16);
                    let decorator_area = child
                        .decorator
                        .as_ref()
                        .map(|d| (d.with(|d| d.width()) + 2) as u16)
                        .unwrap_or(0);
                    let header_area = header_width + padding_width_left;

                    if col_offset <= (total_len + header_area) {
                        if child.value != current_tab {
                            return Some(ClickAction::Title(i, child.value.clone()));
                        }
                        break;
                    }
                    if col_offset < (total_len + header_area + decorator_area) {
                        if let Some(on_decorator_click) = on_decorator_click.as_mut() {
                            return Some(ClickAction::Decorator(
                                i,
                                child.value.clone(),
                                on_decorator_click,
                            ));
                        } else {
                            return Some(ClickAction::Title(i, child.value.clone()));
                        }
                    }
                    total_len += header_area + decorator_area + divider_width + padding_width_right;
                }
                None
            });

            match action {
                Some(ClickAction::Decorator(i, value, on_decorator_click)) => {
                    on_decorator_click(i, &value);
                }
                Some(ClickAction::Title(i, value)) => {
                    on_title_click(i, &value);
                }
                None => {}
            }
        };

        col![
            style(rooibos_reactive::dom::layout::width(width), height(full())),
            wgt!(style(height(header_height)), {
                let headers = Tabs::new(headers())
                    .divider(divider.get())
                    .style(style.get())
                    .highlight_style(Style::default())
                    .select(cur_tab().map(|t| t.1).unwrap_or(0))
                    .padding(padding_left.get(), padding_right.get());
                if let Some(block) = block.get() {
                    headers.block(block)
                } else {
                    headers
                }
            })
            .on_click(on_click)
            .on_key_down(on_key_down)
            .on_direct_focus(on_direct_focus)
            .on_direct_blur(on_direct_blur),
            col![style(max_height(full()), height(body_height)), move || {
                cur_tab().map(|c| c.0()).unwrap_or_else(|| ().into_any())
            }]
        ]
    }
}
