use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::{MaybeProp, MaybeSignal, Signal};
use rooibos_dom::prelude::*;

#[derive(Clone)]
pub struct Tab {
    header: MaybeSignal<Line<'static>>,
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
            children: children.into_children_fn(),
        }
    }
}

pub struct TabView {
    block: MaybeProp<Block<'static>>,
    padding: MaybeProp<u16>,
    highlight_style: MaybeSignal<Style>,
    on_change: Box<dyn FnMut(usize, &str)>,
}

impl Default for TabView {
    fn default() -> Self {
        Self {
            on_change: Box::new(move |_, _| {}),
            padding: Default::default(),
            block: Default::default(),
            highlight_style: Default::default(),
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

    pub fn padding(mut self, padding: impl Into<MaybeProp<u16>>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn highlight_style(mut self, highlight_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.highlight_style = highlight_style.into();
        self
    }

    pub fn on_change(mut self, on_change: impl FnMut(usize, &str) + 'static) -> Self {
        self.on_change = Box::new(on_change);
        self
    }

    pub fn render<S>(
        self,
        current_tab: S,
        children: impl Into<MaybeSignal<Vec<Tab>>>,
    ) -> impl Render
    where
        S: Get<Value = String> + Clone + Send + Sync + 'static,
    {
        let Self {
            block,
            padding,
            highlight_style,
            mut on_change,
        } = self;
        let children: MaybeSignal<Vec<Tab>> = children.into();
        let children = signal!(children.get());
        let headers = {
            signal!(
                children
                    .get()
                    .iter()
                    .map(|t| t.header.get())
                    .collect::<Vec<_>>()
            )
        };
        let current_tab = signal!(current_tab.get());
        let cur_tab = signal!({
            let current_tab = current_tab.get();
            children.get().iter().enumerate().find_map(|(i, c)| {
                if c.value == current_tab {
                    Some((c.children.clone(), i))
                } else {
                    None
                }
            })
        });

        let on_click = move |mouse_event: MouseEvent, event_data: EventData| {
            let start_col = event_data.rect.x;
            let col_offset = mouse_event.column - start_col;
            let children = children.get_untracked();
            let mut total_len = 1u16;
            for (i, child) in children.iter().enumerate() {
                let header = child.header.get_untracked();
                let area = header.width() as u16 + 2;
                if col_offset <= (total_len + area) {
                    if child.value != current_tab.get_untracked() {
                        on_change(i, &child.value);
                    }

                    break;
                }
                total_len += area + 1;
            }
        };

        let length = signal!(padding.get().unwrap_or(0) * 2 + 1);

        col![
            widget_ref!({
                let headers = Tabs::new(headers.get())
                    .select(cur_tab.get().unwrap().1)
                    .highlight_style(highlight_style.get());
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
    }
}
