use ratatui::style::{Style, Styled};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Tabs};
use reactive_graph::traits::{Get, With};
use reactive_graph::wrappers::read::{MaybeProp, MaybeSignal, Signal};
use rooibos_dom::div::taffy::Dimension;
use rooibos_dom::layout::height;
use rooibos_dom::{
    derive_signal, flex_col, line, span, wgt, BlurEvent, ChildrenFn, EventData, FocusEvent,
    IntoAny, IntoChildrenFn, KeyEvent, MouseEvent, Render,
};

use crate::wrapping_list::KeyedWrappingList;
use crate::Keyed;

pub type TabList = KeyedWrappingList<Tab>;

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

impl Keyed for Tab {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.value
    }
}

type OnChangeFn = dyn FnMut(usize, &str);

pub struct TabView {
    block: MaybeProp<Block<'static>>,
    highlight_style: MaybeSignal<Style>,
    decorator_highlight_style: Option<MaybeSignal<Style>>,
    style: MaybeSignal<Style>,
    on_title_click: Box<OnChangeFn>,
    on_decorator_click: Option<Box<OnChangeFn>>,
    on_focus: Box<dyn FnMut(FocusEvent, EventData)>,
    on_blur: Box<dyn FnMut(BlurEvent, EventData)>,
    on_key_down: Box<dyn FnMut(KeyEvent, EventData)>,
    fit: MaybeSignal<bool>,
    divider: MaybeSignal<Span<'static>>,
    header_height: MaybeSignal<Dimension>,
    width: MaybeSignal<Dimension>,
    padding_left: MaybeSignal<Line<'static>>,
    padding_right: MaybeSignal<Line<'static>>,
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
            on_key_down: Box::new(move |_, _| {}),
            on_focus: Box::new(move |_, _| {}),
            on_blur: Box::new(move |_, _| {}),
            block: Default::default(),
            highlight_style: Default::default(),
            decorator_highlight_style: Default::default(),
            style: Default::default(),
            fit: false.into(),
            divider: span!(symbols::line::VERTICAL).into(),
            header_height: Dimension::Length(1.).into(),
            width: Dimension::Percent(100.).into(),
            padding_left: line!(" ").into(),
            padding_right: line!(" ").into(),
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

    pub fn header_height(mut self, header_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        self.header_height = header_height.into();
        self
    }

    pub fn width(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        self.width = width.into();
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
        self.decorator_highlight_style = Some(decorator_highlight_style.into());
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

    pub fn on_focus(mut self, on_focus: impl FnMut(FocusEvent, EventData) + 'static) -> Self {
        self.on_focus = Box::new(on_focus);
        self
    }

    pub fn on_blur(mut self, on_blur: impl FnMut(BlurEvent, EventData) + 'static) -> Self {
        self.on_blur = Box::new(on_blur);
        self
    }

    pub fn on_decorator_click(
        mut self,
        on_decorator_click: impl FnMut(usize, &str) + 'static,
    ) -> Self {
        self.on_decorator_click = Some(Box::new(on_decorator_click));
        self
    }

    pub fn padding_left(mut self, padding_left: impl Into<MaybeSignal<Line<'static>>>) -> Self {
        self.padding_left = padding_left.into();
        self
    }

    pub fn padding_right(mut self, padding_right: impl Into<MaybeSignal<Line<'static>>>) -> Self {
        self.padding_right = padding_right.into();
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
            width,
            fit,
            header_height,
            divider,
            padding_left,
            padding_right,
        } = self;

        let children: MaybeSignal<TabList> = children.into();
        let current_tab: Signal<String> = current_tab.into();

        let cur_tab = {
            let children = children.clone();
            derive_signal!({
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
                                    if let Some(decorator_highlight_style) =
                                        decorator_highlight_style
                                    {
                                        decorator_spans = decorator_spans
                                            .into_iter()
                                            .map(|s| s.set_style(decorator_highlight_style))
                                            .collect();
                                    }
                                }
                                line!([spans, vec![span!("  ")], decorator_spans].concat())
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
            })
        };

        let divider_width = {
            let divider = divider.clone();
            derive_signal!(divider.with(|d| d.width()))
        };

        let padding_width_left = {
            let padding_left = padding_left.clone();
            derive_signal!(padding_left.with(|p| p.width()))
        };

        let padding_width_right = {
            let padding_right = padding_right.clone();
            derive_signal!(padding_right.with(|p| p.width()))
        };

        let headers_len = derive_signal!({
            let headers_len = headers.with(|h| h.len());
            let divider_width = divider_width.get() as u16;

            if headers_len == 0 {
                return 0;
            }
            let padding_width = padding_width_left.get() + padding_width_right.get();
            // title length + padding length + number of dividers (number of tabs - 1)
            // + outside borders (2)
            headers.with(|h| {
                h.iter()
                    .map(|t| (t.width() + padding_width) as u16)
                    .sum::<u16>()
                    + ((headers_len as u16 - 1) * divider_width)
                    + 2
            })
        });

        let width = derive_signal!({
            if fit.get() {
                Dimension::Length(headers_len.get() as f32)
            } else {
                width.get()
            }
        });

        let on_click = move |mouse_event: MouseEvent, event_data: EventData| {
            let start_col = event_data.rect.x;
            let col_offset = mouse_event.column - start_col;

            let divider_width = divider_width.get() as u16;
            let mut total_len = 1u16;
            let current_tab = current_tab.get();
            let padding_width_left = padding_width_left.get() as u16;
            let padding_width_right = padding_width_right.get() as u16;
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

        flex_col![
            props(rooibos_dom::layout::width(width)),
            wgt![props(height(header_height)), {
                let headers = Tabs::new(headers.get())
                    .divider(divider.get())
                    .style(style.get())
                    .highlight_style(Style::default())
                    .select(cur_tab.get().map(|t| t.1).unwrap_or(0))
                    .padding(padding_left.get(), padding_right.get());
                if let Some(block) = block.get() {
                    headers.block(block)
                } else {
                    headers
                }
            }]
            .on_click(on_click)
            .on_key_down(on_key_down)
            .on_focus(on_focus)
            .on_blur(on_blur),
            move || cur_tab
                .get()
                .map(|c| c.0())
                .unwrap_or_else(|| ().into_any())
        ]
    }
}
