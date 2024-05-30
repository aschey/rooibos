use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Paragraph};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Set};
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{
    derive_signal, widget_ref, Constrainable, KeyCode, KeyEvent, Render, WidgetState,
};
use rooibos_runtime::{delay, supports_key_up};

pub struct Button {
    on_click: Rc<RefCell<dyn FnMut()>>,
    constraint: MaybeSignal<Constraint>,
    border_color: MaybeSignal<Color>,
    focused_border_color: MaybeSignal<Color>,
    active_border_color: MaybeSignal<Color>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Constrainable for Button {
    fn constraint<S>(mut self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        self.constraint = constraint.into();
        self
    }
}

impl Button {
    pub fn new() -> Self {
        Self {
            on_click: Rc::new(RefCell::new(|| {})),
            constraint: Default::default(),
            focused_border_color: Color::Blue.into(),
            active_border_color: Color::Green.into(),
            border_color: Color::Gray.into(),
        }
    }

    pub fn border_color(mut self, border_color: impl Into<MaybeSignal<Color>>) -> Self {
        self.border_color = border_color.into();
        self
    }

    pub fn focused_border_color(
        mut self,
        focused_border_color: impl Into<MaybeSignal<Color>>,
    ) -> Self {
        self.focused_border_color = focused_border_color.into();
        self
    }

    pub fn active_border_color(
        mut self,
        active_border_color: impl Into<MaybeSignal<Color>>,
    ) -> Self {
        self.active_border_color = active_border_color.into();
        self
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
        let Self {
            on_click,
            constraint,
            border_color,
            focused_border_color,
            active_border_color,
        } = self;

        let border_type = RwSignal::new(BorderType::Rounded);
        let widget_state = RwSignal::new(WidgetState::Default);

        let border_color = derive_signal!({
            match widget_state.get() {
                WidgetState::Default => border_color.get(),
                WidgetState::Focused => focused_border_color.get(),
                WidgetState::Active => active_border_color.get(),
            }
        });

        let on_enter = move || {
            widget_state.set(WidgetState::Active);

            if !supports_key_up() {
                delay(Duration::from_millis(50), async move {
                    widget_state.set(WidgetState::Focused);
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
                widget_state.set(WidgetState::Focused);
            }
        };
        let children = children.into();
        widget_ref!(
            Paragraph::new(children.get())
                .block(
                    Block::bordered()
                        .border_type(border_type.get())
                        .border_style(Style::default().fg(border_color.get()))
                )
                .centered()
        )
        .constraint(constraint)
        .on_mouse_enter(move |_| border_type.set(BorderType::Double))
        .on_mouse_leave(move |_| border_type.set(BorderType::Rounded))
        .on_click(move |_, _| on_enter_())
        .on_focus(move |_| widget_state.set(WidgetState::Focused))
        .on_blur(move |_| widget_state.set(WidgetState::Default))
        .on_key_down(move |key_event, _| {
            if key_event.code == KeyCode::Enter {
                on_enter()
            }
        })
        .on_key_up(key_up)
    }
}
