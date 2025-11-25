use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState};
use rooibos_dom::events::{
    BlurEvent, ClickEventProps, EventData, EventHandle, FocusEvent, KeyHandler,
};
use rooibos_reactive::dom::{LayoutProps, Render, UpdateLayoutProps};
use rooibos_reactive::graph::IntoReactiveValue;
use rooibos_reactive::graph::traits::{Get, With};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_reactive::wgt;
use tui_theme::Style;

use crate::WrappingList;

type ItemSelectFn<T> = dyn FnMut(usize, &T);

pub struct ListView<T> {
    style: Signal<Style>,
    on_item_click: Box<ItemSelectFn<T>>,
    on_key_down: Box<dyn KeyHandler>,
    on_direct_focus: Box<dyn FnMut(FocusEvent, EventData, EventHandle)>,
    on_direct_blur: Box<dyn FnMut(BlurEvent, EventData, EventHandle)>,
    highlight_style: Signal<Style>,
    block: Option<Signal<Block<'static>>>,
    direction: Signal<ListDirection>,
    highlight_spacing: Signal<HighlightSpacing>,
    highlight_symbol: Option<Signal<&'static str>>,
    repeat_highlight_symbol: Signal<bool>,
    scroll_padding: Signal<usize>,
    layout_props: LayoutProps,
}

impl<T> Default for ListView<T> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            on_item_click: Box::new(move |_, _| {}),
            on_key_down: Box::new(move |_| {}),
            on_direct_focus: Box::new(move |_, _, _| {}),
            on_direct_blur: Box::new(move |_, _, _| {}),
            highlight_style: Style::default().into(),
            block: Default::default(),
            direction: Default::default(),
            highlight_spacing: Default::default(),
            highlight_symbol: Default::default(),
            repeat_highlight_symbol: Default::default(),
            scroll_padding: Default::default(),
            layout_props: LayoutProps::default(),
        }
    }
}

impl<T> UpdateLayoutProps for ListView<T> {
    fn layout_props(&self) -> LayoutProps {
        self.layout_props.clone()
    }

    fn update_props(mut self, props: LayoutProps) -> Self {
        self.layout_props = props;
        self
    }
}

impl<T> ListView<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style<M>(mut self, style: impl IntoReactiveValue<Signal<Style>, M>) -> Self {
        self.style = style.into_reactive_value();
        self
    }

    pub fn on_item_click(mut self, on_item_click: impl FnMut(usize, &T) + 'static) -> Self {
        self.on_item_click = Box::new(on_item_click);
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
        on_blur: impl FnMut(BlurEvent, EventData, EventHandle) + 'static,
    ) -> Self {
        self.on_direct_blur = Box::new(on_blur);
        self
    }

    pub fn highlight_style<M>(
        mut self,
        highlight_style: impl IntoReactiveValue<Signal<Style>, M>,
    ) -> Self {
        self.highlight_style = highlight_style.into_reactive_value();
        self
    }

    pub fn block<M>(mut self, block: impl IntoReactiveValue<Signal<Block<'static>>, M>) -> Self {
        self.block = Some(block.into_reactive_value());
        self
    }

    pub fn direction<M>(
        mut self,
        direction: impl IntoReactiveValue<Signal<ListDirection>, M>,
    ) -> Self {
        self.direction = direction.into_reactive_value();
        self
    }

    pub fn highlight_spacing<M>(
        mut self,
        highlight_spacing: impl IntoReactiveValue<Signal<HighlightSpacing>, M>,
    ) -> Self {
        self.highlight_spacing = highlight_spacing.into_reactive_value();
        self
    }

    pub fn repeat_highlight_symbol<M>(
        mut self,
        repeat_highlight_symbol: impl IntoReactiveValue<Signal<bool>, M>,
    ) -> Self {
        self.repeat_highlight_symbol = repeat_highlight_symbol.into_reactive_value();
        self
    }

    pub fn scroll_padding<M>(
        mut self,
        scroll_padding: impl IntoReactiveValue<Signal<usize>, M>,
    ) -> Self {
        self.scroll_padding = scroll_padding.into_reactive_value();
        self
    }

    pub fn render<M1, M2>(
        self,
        selected: impl IntoReactiveValue<Signal<Option<usize>>, M1>,
        items: impl IntoReactiveValue<Signal<WrappingList<T>>, M2>,
    ) -> impl Render
    where
        T: Into<ListItem<'static>> + Clone + Send + Sync + 'static,
    {
        let Self {
            style,
            mut on_item_click,
            on_key_down,
            on_direct_focus,
            on_direct_blur,
            highlight_style,
            block,
            direction,
            highlight_spacing,
            highlight_symbol,
            repeat_highlight_symbol,
            scroll_padding,
            layout_props,
        } = self;
        let items: Signal<WrappingList<T>> = items.into_reactive_value();
        let selected: Signal<Option<usize>> = selected.into_reactive_value();

        wgt!(ListState::default().with_selected(selected.get()), {
            let mut list = List::new(items.get().0.into_iter().map(Into::into))
                .highlight_style(highlight_style.get())
                .direction(direction.get())
                .highlight_spacing(highlight_spacing.get())
                .repeat_highlight_symbol(repeat_highlight_symbol.get())
                .scroll_padding(scroll_padding.get())
                .style(style.get());
            if let Some(block) = &block {
                list = list.block(block.get());
            }
            if let Some(highlight_symbol) = highlight_symbol {
                list = list.highlight_symbol(highlight_symbol.get());
            }
            list
        })
        .on_click(move |props: ClickEventProps| {
            let clicked_item = items.with(|items| {
                let start_row = props.data.rect.y;
                let row_offset = props.event.row - start_row;
                let mut total_height = 0u16;
                for (i, item) in items.iter().enumerate() {
                    let item_height = item.clone().into().height() as u16;
                    if row_offset < (total_height + item_height) {
                        return Some((i, item.clone()));
                    }
                    total_height += item_height;
                }
                None
            });

            if let Some((i, item)) = clicked_item {
                on_item_click(i, &item);
            }
        })
        .layout_props(layout_props)
        .on_key_down(on_key_down)
        .on_direct_focus(on_direct_focus)
        .on_direct_blur(on_direct_blur)
    }
}
