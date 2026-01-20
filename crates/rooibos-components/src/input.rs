use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::Widget;
use rooibos_dom::events::{
    BlurEvent, EventData, EventHandle, FocusEvent, KeyEventProps, StateChangeEvent,
};
use rooibos_dom::widgets::{Role, WidgetRole};
use rooibos_dom::{
    Event, KeyCode, KeyModifiers, MeasureNode, RenderNode, Repeats, ScrollDirection, set_editing,
};
use rooibos_reactive::IntoSignal;
use rooibos_reactive::dom::div::taffy::Size;
use rooibos_reactive::dom::{DomWidget, LayoutProps, Render, UpdateLayoutProps};
use rooibos_reactive::graph::IntoReactiveValue;
use rooibos_reactive::graph::effect::Effect;
use rooibos_reactive::graph::owner::{StoredValue, on_cleanup};
use rooibos_reactive::graph::signal::{RwSignal, signal};
use rooibos_reactive::graph::traits::{
    Get, GetUntracked, GetValue, Set, Track, Update, UpdateUntracked, With,
};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_theme::Style;
use tokio::sync::broadcast;
use tui_textarea::{CursorMove, TextArea};
use wasm_compat::futures::spawn_local;

use crate::ColorThemeColorTheme;

#[derive(Clone, Copy)]
pub struct InputRef {
    text_area: RwSignal<TextArea<'static>>,
    submit_tx: StoredValue<broadcast::Sender<String>>,
}

impl InputRef {
    pub fn text(&self) -> Signal<String> {
        let text_area = self.text_area;
        (move || text_area.get().lines()[0].clone()).signal()
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
    on_direct_focus: Box<dyn FnMut(FocusEvent, EventData, EventHandle)>,
    on_direct_blur: Box<dyn FnMut(BlurEvent, EventData, EventHandle)>,
    on_state_change: Box<dyn FnMut(StateChangeEvent, EventData, EventHandle)>,
    initial_value: String,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left.into(),
            layout_props: LayoutProps::default(),
            cursor_style: Style::reset().reversed().into(),
            placeholder_style: Style::default().fg_disabled_dark(),
            placeholder_text: String::new().into(),
            style: Style::default().into(),
            on_submit: Box::new(|_| {}),
            on_direct_focus: Box::new(|_, _, _| {}),
            on_direct_blur: Box::new(|_, _, _| {}),
            on_state_change: Box::new(|_, _, _| {}),
            initial_value: "".to_string(),
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
    pub fn placeholder_text<M>(
        mut self,
        placeholder_text: impl IntoReactiveValue<Signal<String>, M>,
    ) -> Self {
        self.placeholder_text = placeholder_text.into_reactive_value();
        self
    }

    pub fn on_submit(mut self, on_submit: impl FnMut(String) + 'static) -> Self {
        self.on_submit = Box::new(on_submit);
        self
    }

    pub fn on_direct_focus(
        mut self,
        on_focus: impl FnMut(FocusEvent, EventData, EventHandle) + 'static,
    ) -> Self {
        self.on_direct_focus = Box::new(on_focus);
        self
    }

    pub fn on_direct_blur(
        mut self,
        on_blur: impl FnMut(BlurEvent, EventData, EventHandle) + 'static,
    ) -> Self {
        self.on_direct_blur = Box::new(on_blur);
        self
    }

    pub fn on_state_change(
        mut self,
        on_state_change: impl FnMut(StateChangeEvent, EventData, EventHandle) + 'static,
    ) -> Self {
        self.on_state_change = Box::new(on_state_change);
        self
    }

    pub fn initial_value(mut self, initial_value: impl Into<String>) -> Self {
        self.initial_value = initial_value.into();
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
            mut on_direct_focus,
            mut on_direct_blur,
            on_state_change,
            initial_value,
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
                t.set_cursor_line_style(Style::default().into());
                t.set_alignment(alignment.get());
                t.set_style(style.get().into());
                t.set_cursor_style(Style::new().into());
                if focused.get() {
                    t.set_cursor_style(cursor_style.get().into());
                } else {
                    // hide cursor when not focused
                    t.set_cursor_style(Style::reset().into());
                }

                t.set_placeholder_text(placeholder_text.get());
                t.set_placeholder_style(placeholder_style.get().into());
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
                    if let Some(event) = to_input(Event::Key(props.event)) {
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

        // TODO: add on_scroll handler
        DomWidget::new(move || {
            text_area.track();
            RenderInput { text_area }
        })
        .layout_props(layout_props)
        .on_key_down(key_down)
        .on_paste(paste)
        .on_direct_focus(move |focus_event, event_data, event_handle| {
            set_editing(true);
            set_focused.set(true);
            on_direct_focus(focus_event, event_data, event_handle);
        })
        .on_state_change(on_state_change)
        .on_direct_blur(move |blur_event, event_data, event_handle| {
            // Notify DOM that we're editing to suppress any quit sequences that could interfere
            set_editing(false);
            set_focused.set(false);
            on_direct_blur(blur_event, event_data, event_handle);
        })
    }
}

fn to_input(event: Event) -> Option<tui_textarea::Input> {
    let (key, modifiers) = to_key(event)?;
    Some(tui_textarea::Input {
        key,
        shift: modifiers.intersects(KeyModifiers::SHIFT),
        ctrl: modifiers.intersects(KeyModifiers::CTRL),
        alt: modifiers.intersects(KeyModifiers::ALT),
    })
}

fn to_key(event: Event) -> Option<(tui_textarea::Key, KeyModifiers)> {
    if let Some((mouse_event, direction)) = event.as_mouse_scroll() {
        match direction {
            ScrollDirection::Up => {
                return Some((tui_textarea::Key::MouseScrollUp, mouse_event.modifiers));
            }
            ScrollDirection::Down => {
                return Some((tui_textarea::Key::MouseScrollDown, mouse_event.modifiers));
            }
            _ => {}
        }
    }

    let key_event = event.as_key_press(Repeats::Include)?;

    let key_code = match key_event.code {
        KeyCode::Backspace => tui_textarea::Key::Backspace,
        KeyCode::Enter => tui_textarea::Key::Enter,
        KeyCode::Left => tui_textarea::Key::Left,
        KeyCode::Right => tui_textarea::Key::Right,
        KeyCode::Up => tui_textarea::Key::Up,
        KeyCode::Down => tui_textarea::Key::Down,
        KeyCode::Home => tui_textarea::Key::Home,
        KeyCode::End => tui_textarea::Key::End,
        KeyCode::PageUp => tui_textarea::Key::PageUp,
        KeyCode::PageDown => tui_textarea::Key::PageDown,
        KeyCode::Tab => tui_textarea::Key::Tab,
        KeyCode::Delete => tui_textarea::Key::Delete,
        KeyCode::F(f) => tui_textarea::Key::F(f),
        KeyCode::Char(c) => tui_textarea::Key::Char(c),
        KeyCode::Esc => tui_textarea::Key::Esc,
        _ => return None,
    };
    Some((key_code, key_event.modifiers))
}

struct RenderInput {
    text_area: RwSignal<TextArea<'static>>,
}

impl WidgetRole for RenderInput {
    fn widget_role() -> Option<Role> {
        Some(Role::TextInput)
    }
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
