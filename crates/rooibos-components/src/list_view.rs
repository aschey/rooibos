use ratatui::style::Style;
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState};
use rooibos_dom::{BlurEvent, EventData, EventHandle, FocusEvent, KeyEvent};
use rooibos_reactive::graph::traits::{Get, With};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;
use rooibos_reactive::{wgt, Render};

use crate::WrappingList;

type ItemSelectFn<T> = dyn FnMut(usize, &T);

pub struct ListView<T> {
    style: MaybeSignal<Style>,
    on_item_click: Box<ItemSelectFn<T>>,
    on_key_down: Box<dyn FnMut(KeyEvent, EventData, EventHandle)>,
    on_focus: Box<dyn FnMut(FocusEvent, EventData)>,
    on_blur: Box<dyn FnMut(BlurEvent, EventData)>,
    highlight_style: MaybeSignal<Style>,
    block: Option<MaybeSignal<Block<'static>>>,
    direction: MaybeSignal<ListDirection>,
    highlight_spacing: MaybeSignal<HighlightSpacing>,
    highlight_symbol: Option<MaybeSignal<&'static str>>,
    repeat_highlight_symbol: MaybeSignal<bool>,
    scroll_padding: MaybeSignal<usize>,
}

impl<T> Default for ListView<T> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            on_item_click: Box::new(move |_, _| {}),
            on_key_down: Box::new(move |_, _, _| {}),
            on_focus: Box::new(move |_, _| {}),
            on_blur: Box::new(move |_, _| {}),
            highlight_style: Style::default().into(),
            block: Default::default(),
            direction: Default::default(),
            highlight_spacing: Default::default(),
            highlight_symbol: Default::default(),
            repeat_highlight_symbol: Default::default(),
            scroll_padding: Default::default(),
        }
    }
}

impl<T> ListView<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: impl Into<MaybeSignal<Style>>) -> Self {
        self.style = style.into();
        self
    }

    pub fn on_item_click(mut self, on_item_click: impl FnMut(usize, &T) + 'static) -> Self {
        self.on_item_click = Box::new(on_item_click);
        self
    }

    pub fn on_key_down(
        mut self,
        on_key_down: impl FnMut(KeyEvent, EventData, EventHandle) + 'static,
    ) -> Self {
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

    pub fn highlight_style(mut self, highlight_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.highlight_style = highlight_style.into();
        self
    }

    pub fn block(mut self, block: impl Into<MaybeSignal<Block<'static>>>) -> Self {
        self.block = Some(block.into());
        self
    }

    pub fn direction(mut self, direction: impl Into<MaybeSignal<ListDirection>>) -> Self {
        self.direction = direction.into();
        self
    }

    pub fn highlight_spacing(
        mut self,
        highlight_spacing: impl Into<MaybeSignal<HighlightSpacing>>,
    ) -> Self {
        self.highlight_spacing = highlight_spacing.into();
        self
    }

    pub fn repeat_highlight_symbol(
        mut self,
        repeat_highlight_symbol: impl Into<MaybeSignal<bool>>,
    ) -> Self {
        self.repeat_highlight_symbol = repeat_highlight_symbol.into();
        self
    }

    pub fn scroll_padding(mut self, scroll_padding: impl Into<MaybeSignal<usize>>) -> Self {
        self.scroll_padding = scroll_padding.into();
        self
    }

    pub fn render(
        self,
        selected: impl Into<MaybeSignal<Option<usize>>>,
        items: impl Into<MaybeSignal<WrappingList<T>>>,
    ) -> impl Render
    where
        T: Into<ListItem<'static>> + Clone + Send + Sync + 'static,
    {
        let Self {
            style,
            mut on_item_click,
            on_key_down,
            on_focus,
            on_blur,
            highlight_style,
            block,
            direction,
            highlight_spacing,
            highlight_symbol,
            repeat_highlight_symbol,
            scroll_padding,
        } = self;
        let items: MaybeSignal<WrappingList<T>> = items.into();
        let selected: MaybeSignal<Option<usize>> = selected.into();
        {
            let items = items.clone();
            wgt!(
                {
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
                },
                ListState::default().with_selected(selected.get())
            )
        }
        .on_click(move |mouse_event, event_data, _| {
            let clicked_item = items.with(|items| {
                let start_row = event_data.rect.y;
                let row_offset = mouse_event.row - start_row;
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
        .on_key_down(on_key_down)
        .on_focus(on_focus)
        .on_blur(on_blur)
    }
}
