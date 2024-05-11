use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Paragraph};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Set};
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{widget_ref, KeyCode, KeyEvent, Render};
use rooibos_runtime::{delay, supports_key_up};

pub struct Button {
    on_click: Rc<RefCell<dyn FnMut()>>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    pub fn new() -> Self {
        Self {
            on_click: Rc::new(RefCell::new(|| {})),
        }
    }

    pub fn on_click<C>(mut self, on_click: C) -> Self
    where
        C: FnMut() + Clone + 'static,
    {
        self.on_click = Rc::new(RefCell::new(on_click));
        self
    }

    pub fn render<M>(self, children: M) -> impl Render
    where
        M: Into<MaybeSignal<Text<'static>>> + 'static,
    {
        let Self { on_click } = self;
        let border_color = RwSignal::new(Color::Gray);

        let on_enter = move || {
            border_color.set(Color::Green);

            if !supports_key_up() {
                delay(Duration::from_millis(50), async move {
                    border_color.set(Color::Blue);
                });
            }
            on_click.borrow_mut()()
        };

        let on_enter_ = on_enter.clone();

        let key_up = move |key_event: KeyEvent, _| {
            if !supports_key_up() {
                return;
            }
            if key_event.code == KeyCode::Enter {
                border_color.set(Color::Blue);
            }
        };
        let children = children.into();
        widget_ref!(
            Paragraph::new(children.get())
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(border_color.get()))
                )
                .centered()
        )
        .focusable(true)
        .on_click(move |_, _| on_enter_())
        .on_focus(move |_| border_color.set(Color::Blue))
        .on_blur(move |_| border_color.set(Color::Gray))
        .on_key_down(move |key_event, _| {
            if key_event.code == KeyCode::Enter {
                on_enter()
            }
        })
        .on_key_up(key_up)
    }
}
