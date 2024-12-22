use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::Widget;
use rooibos_dom::events::{BlurEvent, EventData, FocusEvent, KeyEventProps};
use rooibos_dom::{KeyCode, KeyEvent, MeasureNode, NodeId, RenderNode, set_editing};
use rooibos_reactive::derive_signal;
use rooibos_reactive::dom::div::taffy::Size;
use rooibos_reactive::dom::{DomWidget, LayoutProps, Render, UpdateLayoutProps};
use rooibos_reactive::graph::effect::Effect;
use rooibos_reactive::graph::owner::{StoredValue, on_cleanup};
use rooibos_reactive::graph::signal::{RwSignal, signal};
use rooibos_reactive::graph::traits::{
    Get, GetUntracked, GetValue, Set, Track, Update, UpdateUntracked, With,
};
use rooibos_reactive::graph::wrappers::read::Signal;
use tokio::sync::broadcast;
use tui_textarea::{CursorMove, TextArea};
use wasm_compat::futures::spawn_local;

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

    pub fn delete_line(&self) -> bool {
        self.text_area
            .try_update(|t| {
                t.move_cursor(CursorMove::End);
                t.delete_line_by_head()
            })
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
    layout_props: LayoutProps,
    alignment: Signal<Alignment>,
    cursor_style: Signal<Style>,
    style: Signal<Style>,
    placeholder_style: Signal<Style>,
    placeholder_text: Signal<String>,
    on_submit: Box<dyn FnMut(String)>,
    on_focus: Box<dyn FnMut(FocusEvent, EventData)>,
    on_blur: Box<dyn FnMut(BlurEvent, EventData)>,
    initial_value: String,
    id: Option<NodeId>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left.into(),
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
    pub fn placeholder_text(mut self, placeholder_text: impl Into<Signal<String>>) -> Self {
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

        on_cleanup(|| {
            set_editing(false);
        });

        text_area.update_untracked(|t| {
            t.delete_line_by_head();
            t.insert_str(initial_value);
        });

        let (focused, set_focused) = signal(false);

        spawn_local(async move {
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
                if focused.get() {
                    t.set_cursor_style(cursor_style.get());
                } else {
                    // hide cursor when not focused
                    t.set_cursor_style(Style::reset());
                }

                t.set_placeholder_text(placeholder_text.get());
                t.set_placeholder_style(placeholder_style.get());
            });
        });

        let key_down = {
            move |mut props: KeyEventProps| {
                let has_modifiers = !props.event.modifiers.is_empty();
                if !has_modifiers {
                    // If the input widget is focused and there's no modifiers, we should consume
                    // the input to prevent it from triggering any keybindings
                    // on a parent
                    props.handle.stop_propagation();
                }
                if props.event.code == KeyCode::Enter && !has_modifiers {
                    let line = text_area.with(|t| t.lines()[0].clone());
                    submit_tx.send(line).unwrap();
                    return;
                }

                text_area.update(|t| {
                    #[cfg(feature = "crossterm")]
                    if let Ok(event) =
                        <KeyEvent as TryInto<crossterm::event::KeyEvent>>::try_into(props.event)
                    {
                        t.input(event);
                    }
                });
            }
        };

        let paste = {
            move |text: String, _, _| {
                text_area.update(|t| {
                    t.insert_str(text);
                });
            }
        };

        let mut widget = DomWidget::new::<TextArea, _>(move || {
            text_area.track();
            RenderInput { text_area }
        })
        .layout_props(layout_props)
        .on_key_down(key_down)
        .on_paste(paste)
        .on_focus(move |focus_event, event_data| {
            set_editing(true);
            set_focused.set(true);
            on_focus(focus_event, event_data);
        })
        .on_blur(move |blur_event, event_data| {
            // Notify DOM that we're editing to suppress any quit sequences that could interfere
            set_editing(false);
            set_focused.set(false);
            on_blur(blur_event, event_data);
        });
        if let Some(id) = id {
            widget = widget.id(id);
        }
        widget
    }
}

struct RenderInput {
    text_area: RwSignal<TextArea<'static>>,
}

impl RenderNode for RenderInput {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        self.text_area.with(|t| t.render(area, frame.buffer_mut()));
    }
}

impl MeasureNode for RenderInput {
    fn measure(
        &self,
        known_dimensions: rooibos_reactive::dom::div::taffy::Size<Option<f32>>,
        available_space: rooibos_reactive::dom::div::taffy::Size<
            rooibos_reactive::dom::div::taffy::AvailableSpace,
        >,
        style: &rooibos_reactive::dom::div::taffy::Style,
    ) -> rooibos_reactive::dom::div::taffy::Size<f32> {
        let text = self.text_area.get_untracked();
        let lines = text.lines();
        let max_len = lines
            .iter()
            .map(|l| l.measure(known_dimensions, available_space, style).width)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.);

        Size {
            // +1 for cursor
            width: (text.placeholder_text().len() as f32).max(max_len) + 1.,
            height: lines.len() as f32,
        }
    }

    fn estimate_size(&self) -> Size<f32> {
        let text = self.text_area.get_untracked();
        let lines = text.lines();
        let max_len = lines
            .iter()
            .map(|l| l.estimate_size().width)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.);

        Size {
            // +1 for cursor
            width: (text.placeholder_text().len() as f32).max(max_len) + 1.,
            height: lines.len() as f32,
        }
    }
}
