use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Widget};
use reactive_graph::effect::Effect;
use reactive_graph::owner::StoredValue;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Set, Track, Update, UpdateUntracked, With};
use reactive_graph::wrappers::read::{MaybeSignal, Signal};
use rooibos_dom::{
    derive_signal, BlurEvent, Constrainable, DomWidget, EventData, FocusEvent, KeyCode, KeyEvent,
    LayoutProps, NodeId, Render, UpdateLayoutProps, WidgetState,
};
use rooibos_runtime::wasm_compat;
use tokio::sync::broadcast;
use tui_textarea::{CursorMove, TextArea};

#[derive(Clone, Copy)]
pub struct InputRef {
    text_area: RwSignal<TextArea<'static>>,
    submit_tx: StoredValue<broadcast::Sender<String>>,
}

impl InputRef {
    pub fn text(&self) -> Signal<String> {
        let text_area = self.text_area;
        derive_signal!(text_area.get().lines()[0].clone())
    }
}

impl InputRef {
    pub fn submit(&self) {
        self.submit_tx
            .get_value()
            .send(self.text_area.with(|t| t.lines()[0].to_owned()))
            .unwrap();
    }

    pub fn cancel_selection(&self) {
        self.text_area.update(|t| {
            t.cancel_selection();
        })
    }

    pub fn set_mask_char(&self, mask: char) {
        self.text_area.update(|t| {
            t.set_mask_char(mask);
        })
    }

    pub fn clear_mask_char(&self) {
        self.text_area.update(|t| {
            t.clear_mask_char();
        })
    }

    pub fn cut(&self) -> bool {
        self.text_area.try_update(|t| t.cut()).unwrap()
    }

    pub fn paste(&self) -> bool {
        self.text_area.try_update(|t| t.paste()).unwrap()
    }

    pub fn move_cursor(&self, cursor_move: CursorMove) {
        self.text_area.update(|t| t.move_cursor(cursor_move));
    }

    pub fn delete_line_by_head(&self) -> bool {
        self.text_area
            .try_update(|t| t.delete_line_by_head())
            .unwrap()
    }

    pub fn delete_line_by_end(&self) -> bool {
        self.text_area
            .try_update(|t| t.delete_line_by_end())
            .unwrap()
    }

    pub fn with_lines<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&[String]) -> T,
    {
        self.text_area.with(|t| f(t.lines()))
    }

    pub fn set_hard_tab_indent(&self, set: bool) {
        self.text_area.update(|t| t.set_hard_tab_indent(set));
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.text_area.with(|t| t.cursor())
    }

    #[cfg(feature = "input-search")]
    pub fn search_forward(&self, match_cursor: bool) -> bool {
        self.text_area
            .try_update(|t| t.search_forward(match_cursor))
            .unwrap()
    }

    #[cfg(feature = "input-search")]
    pub fn search_back(&self, match_cursor: bool) -> bool {
        self.text_area
            .try_update(|t| t.search_back(match_cursor))
            .unwrap()
    }
}

pub struct Input {
    // constraint: MaybeSignal<Constraint>,
    layout_props: LayoutProps,
    alignment: MaybeSignal<Alignment>,
    block: Box<dyn Fn(WidgetState) -> Option<Block<'static>> + Send + Sync>,
    cursor_style: MaybeSignal<Style>,
    style: MaybeSignal<Style>,
    placeholder_style: MaybeSignal<Style>,
    placeholder_text: MaybeSignal<String>,
    on_submit: Box<dyn FnMut(String)>,
    on_focus: Box<dyn FnMut(FocusEvent, EventData)>,
    on_blur: Box<dyn FnMut(BlurEvent, EventData)>,
    initial_value: String,
    id: Option<NodeId>,
}

impl Constrainable for Input {
    type Output = Self;

    fn constraint<S>(self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        // self.constraint = constraint.into();
        self
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left.into(),
            block: Box::new(move |_| None),
            layout_props: LayoutProps::default(),
            cursor_style: Style::reset().reversed().into(),
            placeholder_style: Style::default().dark_gray().into(),
            placeholder_text: String::new().into(),
            style: Style::default().into(),
            on_submit: Box::new(|_| {}),
            on_focus: Box::new(|_, _| {}),
            on_blur: Box::new(|_, _| {}),
            initial_value: "".to_string(),
            id: None,
        }
    }
}

impl UpdateLayoutProps for Input {
    fn layout_props(&self) -> LayoutProps {
        self.layout_props.clone()
    }

    fn update_props(mut self, props: LayoutProps) -> Self {
        self.layout_props = props;
        self
    }
}

impl Input {
    pub fn block(
        mut self,
        block: impl Fn(WidgetState) -> Option<Block<'static>> + Send + Sync + 'static,
    ) -> Self {
        self.block = Box::new(block);
        self
    }

    pub fn placeholder_text(mut self, placeholder_text: impl Into<MaybeSignal<String>>) -> Self {
        self.placeholder_text = placeholder_text.into();
        self
    }

    pub fn on_submit(mut self, on_submit: impl FnMut(String) + 'static) -> Self {
        self.on_submit = Box::new(on_submit);
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

    pub fn initial_value(mut self, initial_value: impl Into<String>) -> Self {
        self.initial_value = initial_value.into();
        self
    }

    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn get_ref() -> InputRef {
        let (submit_tx, _) = broadcast::channel(32);
        InputRef {
            text_area: RwSignal::new(TextArea::default()),
            submit_tx: StoredValue::new(submit_tx),
        }
    }

    pub fn render(self, input_ref: InputRef) -> impl Render {
        let Self {
            layout_props,
            alignment,
            block,
            cursor_style,
            placeholder_style,
            style,
            placeholder_text,
            mut on_submit,
            mut on_focus,
            mut on_blur,
            initial_value,
            id,
        } = self;

        let text_area = input_ref.text_area;
        let submit_tx = input_ref.submit_tx.get_value();
        let mut submit_rx = submit_tx.subscribe();

        text_area.update_untracked(|t| {
            t.delete_line_by_head();
            t.insert_str(initial_value);
        });

        let widget_state = RwSignal::new(WidgetState::Default);
        let block = derive_signal!({
            let state = widget_state.get();
            return block(state);
        });

        wasm_compat::spawn_local(async move {
            while let Ok(line) = submit_rx.recv().await {
                on_submit(line);
            }
        });

        Effect::new(move || {
            text_area.update(|t| {
                t.set_cursor_line_style(Style::default());
                t.set_alignment(alignment.get());
                t.set_style(style.get());
                t.set_cursor_style(Style::new());
                if widget_state.get() == WidgetState::Focused {
                    t.set_cursor_style(cursor_style.get());
                } else {
                    // hide cursor when not focused
                    t.set_cursor_style(Style::reset());
                }

                t.set_placeholder_text(placeholder_text.get());
                t.set_placeholder_style(placeholder_style.get());
                if let Some(block) = block.get() {
                    t.set_block(block)
                }
            });
        });

        let key_down = move |key_event: KeyEvent, _| {
            if key_event.code == KeyCode::Enter && key_event.modifiers.is_empty() {
                let line = text_area.with(|t| t.lines()[0].clone());
                submit_tx.send(line).unwrap();
                return;
            }

            text_area.update(|t| {
                #[cfg(feature = "crossterm")]
                if let Ok(event) =
                    <KeyEvent as TryInto<crossterm::event::KeyEvent>>::try_into(key_event)
                {
                    t.input(event);
                }
            });
        };

        let paste = move |text: String, _| {
            text_area.update(|t| {
                t.insert_str(text);
            });
        };

        let mut widget = DomWidget::new::<TextArea, _, _>(move || {
            text_area.track();
            move |area: Rect, buf: &mut Buffer| {
                text_area.with(|t| t.render(area, buf));
            }
        })
        .layout_props(layout_props)
        .on_key_down(key_down)
        .on_paste(paste)
        .on_focus(move |focus_event, event_data| {
            widget_state.set(WidgetState::Focused);
            on_focus(focus_event, event_data);
        })
        .on_blur(move |blur_event, event_data| {
            widget_state.set(WidgetState::Default);
            on_blur(blur_event, event_data);
        });
        if let Some(id) = id {
            widget = widget.id(id);
        }
        widget
    }
}
