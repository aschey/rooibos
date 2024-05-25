use ratatui::style::Style;
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState};
use reactive_graph::traits::{Get, With};
use reactive_graph::wrappers::read::{MaybeProp, MaybeSignal};
use rooibos_dom::{stateful_widget, EventData, KeyEvent, Render};

use crate::WrappingList;

type ItemSelectFn = dyn FnMut(usize, &ListItem);

pub type ListItems<'a> = WrappingList<ListItem<'a>>;

pub struct ListView {
    style: MaybeSignal<Style>,
    on_item_click: Box<ItemSelectFn>,
    on_key_down: Box<dyn FnMut(KeyEvent, EventData)>,
    highlight_style: MaybeSignal<Style>,
    block: MaybeProp<Block<'static>>,
    direction: MaybeSignal<ListDirection>,
    highlight_spacing: MaybeSignal<HighlightSpacing>,
    highlight_symbol: MaybeProp<&'static str>,
    repeat_highlight_symbol: MaybeSignal<bool>,
    scroll_padding: MaybeSignal<usize>,
}

impl Default for ListView {
    fn default() -> Self {
        Self {
            style: Default::default(),
            on_item_click: Box::new(move |_, _| {}),
            on_key_down: Box::new(move |_, _| {}),
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

impl ListView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: impl Into<MaybeSignal<Style>>) -> Self {
        self.style = style.into();
        self
    }

    pub fn on_item_click(mut self, on_item_click: impl FnMut(usize, &ListItem) + 'static) -> Self {
        self.on_item_click = Box::new(on_item_click);
        self
    }

    pub fn on_key_down(mut self, on_key_down: impl FnMut(KeyEvent, EventData) + 'static) -> Self {
        self.on_key_down = Box::new(on_key_down);
        self
    }

    pub fn highlight_style(mut self, highlight_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.highlight_style = highlight_style.into();
        self
    }

    pub fn block(mut self, block: impl Into<MaybeProp<Block<'static>>>) -> Self {
        self.block = block.into();
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
        items: impl Into<MaybeSignal<ListItems<'static>>>,
    ) -> impl Render {
        let Self {
            style,
            mut on_item_click,
            on_key_down,
            highlight_style,
            block,
            direction,
            highlight_spacing,
            highlight_symbol,
            repeat_highlight_symbol,
            scroll_padding,
        } = self;
        let items: MaybeSignal<ListItems> = items.into();
        let selected: MaybeSignal<Option<usize>> = selected.into();
        {
            let items = items.clone();
            stateful_widget!(
                {
                    let mut list = List::new(items.get().0)
                        .highlight_style(highlight_style.get())
                        .direction(direction.get())
                        .highlight_spacing(highlight_spacing.get())
                        .repeat_highlight_symbol(repeat_highlight_symbol.get())
                        .scroll_padding(scroll_padding.get())
                        .style(style.get());
                    if let Some(block) = block.get() {
                        list = list.block(block);
                    }
                    if let Some(highlight_symbol) = highlight_symbol.get() {
                        list = list.highlight_symbol(highlight_symbol);
                    }
                    list
                },
                ListState::default().with_selected(selected.get())
            )
        }
        .on_click(move |mouse_event, event_data| {
            let clicked_item = items.with(|items| {
                let start_row = event_data.rect.y;
                let row_offset = mouse_event.row - start_row;
                let mut total_height = 0u16;
                for (i, item) in items.iter().enumerate() {
                    let item_height = item.height() as u16;
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
    }
}
