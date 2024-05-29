use std::any::type_name;

use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::Block;
use ratatui::Frame;
use reactive_graph::effect::RenderEffect;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Update, UpdateUntracked, With};
use reactive_graph::wrappers::read::{MaybeSignal, Signal};
use rooibos_dom::{derive_signal, Constrainable, DomWidget, KeyCode, KeyEvent, Render};
use tui_textarea::{CursorMove, TextArea};

#[derive(Clone, Copy)]
pub struct InputRef(RwSignal<TextArea<'static>>);

impl InputRef {
    pub fn text(&self) -> Signal<String> {
        let textarea = self.0;
        derive_signal!(textarea.get().lines()[0].clone())
    }
}

impl InputRef {
    pub fn cancel_selection(&self) {
        self.0.update(|t| {
            t.cancel_selection();
        })
    }

    pub fn set_mask_char(&self, mask: char) {
        self.0.update(|t| {
            t.set_mask_char(mask);
        })
    }

    pub fn clear_mask_char(&self) {
        self.0.update(|t| {
            t.clear_mask_char();
        })
    }

    pub fn cut(&self) -> bool {
        self.0.try_update(|t| t.cut()).unwrap()
    }

    pub fn paste(&self) -> bool {
        self.0.try_update(|t| t.paste()).unwrap()
    }

    pub fn move_cursor(&self, cursor_move: CursorMove) {
        self.0.update(|t| t.move_cursor(cursor_move));
    }

    pub fn delete_line_by_head(&self) -> bool {
        self.0.try_update(|t| t.delete_line_by_head()).unwrap()
    }

    pub fn delete_line_by_end(&self) -> bool {
        self.0.try_update(|t| t.delete_line_by_end()).unwrap()
    }

    pub fn with_lines<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&[String]) -> T,
    {
        self.0.with(|t| f(t.lines()))
    }

    pub fn set_hard_tab_indent(&self, set: bool) {
        self.0.update(|t| t.set_hard_tab_indent(set));
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.0.with(|t| t.cursor())
    }

    #[cfg(feature = "input-search")]
    pub fn search_forward(&self, match_cursor: bool) -> bool {
        self.0
            .try_update(|t| t.search_forward(match_cursor))
            .unwrap()
    }

    #[cfg(feature = "input-search")]
    pub fn search_back(&self, match_cursor: bool) -> bool {
        self.0.try_update(|t| t.search_back(match_cursor)).unwrap()
    }
}

pub struct Input {
    constraint: MaybeSignal<Constraint>,
    alignment: MaybeSignal<Alignment>,
    block: Option<MaybeSignal<Block<'static>>>,
    cursor_style: MaybeSignal<Style>,
    style: MaybeSignal<Style>,
    placeholder_style: MaybeSignal<Style>,
    placeholder_text: MaybeSignal<String>,
    on_submit: Box<dyn FnMut(String)>,
    initial_value: String,
}

impl Constrainable for Input {
    fn constraint<S>(mut self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        self.constraint = constraint.into();
        self
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left.into(),
            block: None,
            constraint: Constraint::default().into(),
            cursor_style: Style::default().reversed().into(),
            placeholder_style: Style::default().dark_gray().into(),
            placeholder_text: String::new().into(),
            style: Style::default().into(),
            on_submit: Box::new(|_| {}),
            initial_value: "".to_string(),
        }
    }
}

impl Input {
    pub fn block(mut self, block: impl Into<MaybeSignal<Block<'static>>>) -> Self {
        self.block = Some(block.into());
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

    pub fn initial_value(mut self, initial_value: impl Into<String>) -> Self {
        self.initial_value = initial_value.into();
        self
    }

    pub fn get_ref() -> InputRef {
        InputRef(RwSignal::new(TextArea::default()))
    }

    pub fn render(self, input_ref: InputRef) -> impl Render {
        let Self {
            constraint,
            alignment,
            block,
            cursor_style,
            placeholder_style,
            style,
            placeholder_text,
            mut on_submit,
            initial_value,
        } = self;
        let text_area = input_ref.0;
        text_area.update_untracked(|t| t.insert_str(initial_value));

        RenderEffect::new(move |_| {
            text_area.update(|t| {
                t.set_cursor_line_style(Style::default());
                t.set_alignment(alignment.get());
                t.set_style(style.get());
                t.set_cursor_style(cursor_style.get());
                t.set_placeholder_text(placeholder_text.get());
                t.set_placeholder_style(placeholder_style.get());
                if let Some(block) = block.clone() {
                    t.set_block(block.get())
                }
            });
        });

        let key_down = move |key_event: KeyEvent, _| {
            if key_event.code == KeyCode::Enter && key_event.modifiers.is_empty() {
                let line = text_area.with(|t| t.lines()[0].clone());
                on_submit(line);
                return;
            }

            text_area.update(|t| {
                #[cfg(feature = "crossterm")]
                let event: crossterm::event::KeyEvent = key_event.into();
                t.input(event);
            });
        };

        DomWidget::new(type_name::<TextArea>(), move || {
            let text_area = text_area.get();
            move |f: &mut Frame, area: Rect| {
                f.render_widget(text_area.widget(), area);
            }
        })
        .constraint(constraint)
        .on_key_down(key_down)
    }
}
