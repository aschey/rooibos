use ratatui::style::Style;
use ratatui::widgets::{List, ListItem, ListState};
use reactive_graph::traits::{Get, With};
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{stateful_widget, Render};

type ItemSelectFn = dyn FnMut(usize, &ListItem);
pub struct ListView {
    on_item_click: Box<ItemSelectFn>,
    highlight_style: MaybeSignal<Style>,
}

impl Default for ListView {
    fn default() -> Self {
        Self {
            on_item_click: Box::new(move |_, _| {}),
            highlight_style: Style::default().into(),
        }
    }
}

impl ListView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_item_click(mut self, on_item_click: impl FnMut(usize, &ListItem) + 'static) -> Self {
        self.on_item_click = Box::new(on_item_click);
        self
    }

    pub fn highlight_style(mut self, highlight_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.highlight_style = highlight_style.into();
        self
    }

    pub fn render(
        self,
        selected: impl Into<MaybeSignal<Option<usize>>>,
        items: impl Into<MaybeSignal<Vec<ListItem<'static>>>>,
    ) -> impl Render {
        let Self {
            mut on_item_click,
            highlight_style,
        } = self;
        let items: MaybeSignal<Vec<ListItem<'static>>> = items.into();
        let selected: MaybeSignal<Option<usize>> = selected.into();
        {
            let items = items.clone();
            stateful_widget!(
                List::new(items.get()).highlight_style(highlight_style.get()),
                ListState::default().with_selected(selected.get())
            )
        }
        .focusable(true)
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
    }
}
