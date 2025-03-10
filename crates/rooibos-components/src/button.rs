use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::layout::Alignment;
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use rooibos_dom::events::KeyEventProps;
use rooibos_dom::{KeyCode, delay, supports_keyboard_enhancement};
use rooibos_reactive::dom::layout::{BorderType, Borders, borders};
use rooibos_reactive::dom::{LayoutProps, Render, UpdateLayoutProps};
use rooibos_reactive::graph::owner::StoredValue;
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, GetValue, Set, WithValue};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_reactive::{derive_signal, wgt};
use tokio::sync::broadcast;
use tokio::task::spawn_local;

#[derive(Clone, Copy)]
pub struct ButtonRef {
    tx: StoredValue<broadcast::Sender<()>>,
}

impl Default for ButtonRef {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonRef {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            tx: StoredValue::new(tx),
        }
    }
    pub fn click(&self) {
        self.tx.with_value(|tx| tx.send(()).unwrap());
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WidgetState {
    Focused,
    Active,
    Default,
}

pub struct Button {
    on_click: Rc<RefCell<dyn FnMut()>>,
    layout_props: LayoutProps,
    border_color: Signal<Color>,
    focused_border_color: Signal<Color>,
    active_border_color: Signal<Color>,
    element_ref: Option<ButtonRef>,
    text_alignment: Signal<Alignment>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateLayoutProps for Button {
    fn layout_props(&self) -> LayoutProps {
        self.layout_props.clone()
    }

    fn update_props(mut self, props: LayoutProps) -> Self {
        self.layout_props = props;
        self
    }
}

impl Button {
    pub fn new() -> Self {
        Self {
            on_click: Rc::new(RefCell::new(|| {})),
            layout_props: LayoutProps::default(),
            focused_border_color: Color::Blue.into(),
            active_border_color: Color::Green.into(),
            border_color: Color::Gray.into(),
            text_alignment: Alignment::Left.into(),
            element_ref: None,
        }
    }

    pub fn element_ref(mut self, button_ref: ButtonRef) -> Self {
        self.element_ref = Some(button_ref);
        self
    }

    pub fn border_color(mut self, border_color: impl Into<Signal<Color>>) -> Self {
        self.border_color = border_color.into();
        self
    }

    pub fn focused_border_color(mut self, focused_border_color: impl Into<Signal<Color>>) -> Self {
        self.focused_border_color = focused_border_color.into();
        self
    }

    pub fn active_border_color(mut self, active_border_color: impl Into<Signal<Color>>) -> Self {
        self.active_border_color = active_border_color.into();
        self
    }

    pub fn text_alignment(mut self, alignment: impl Into<Signal<Alignment>>) -> Self {
        self.text_alignment = alignment.into();
        self
    }

    pub fn left_aligned(self) -> Self {
        self.text_alignment(Alignment::Left)
    }

    pub fn right_aligned(self) -> Self {
        self.text_alignment(Alignment::Right)
    }

    pub fn centered(self) -> Self {
        self.text_alignment(Alignment::Center)
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
        M: Into<Signal<Text<'static>>> + 'static,
    {
        let Self {
            on_click,
            layout_props,
            border_color,
            focused_border_color,
            active_border_color,
            element_ref,
            text_alignment,
        } = self;
        let enabled = layout_props.enabled.value().unwrap_or(true.into());

        let border_type = RwSignal::new(BorderType::Round);
        let focused = RwSignal::new(false);
        let active = RwSignal::new(false);

        let widget_state = derive_signal!(if active.get() {
            WidgetState::Active
        } else if focused.get() {
            WidgetState::Focused
        } else {
            WidgetState::Default
        });

        let current_border_color = derive_signal!({
            match widget_state.get() {
                WidgetState::Default => border_color.get(),
                WidgetState::Focused => focused_border_color.get(),
                WidgetState::Active => active_border_color.get(),
            }
        });

        let on_enter = move || {
            active.set(true);
            if !supports_keyboard_enhancement() {
                delay(Duration::from_millis(50), async move {
                    // Need to use try_set here in case the button was already disposed
                    active.try_set(false);
                });
            }
            on_click.borrow_mut()()
        };

        if let Some(element_ref) = element_ref {
            let tx = element_ref.tx.get_value();
            let mut rx = tx.subscribe();
            let on_enter = on_enter.clone();
            spawn_local(async move {
                while let Ok(()) = rx.recv().await {
                    if enabled.get() {
                        on_enter();
                    }
                }
            });
        }

        let key_up = move |props: KeyEventProps| {
            if !supports_keyboard_enhancement() {
                return;
            }
            if props.event.code == KeyCode::Enter {
                focused.set(true);
            }
        };

        let button_borders = derive_signal!(if enabled.get() {
            Borders::all()
                .outer()
                .border_type(border_type.get())
                .fg(current_border_color.get())
        } else {
            Borders::all().inner().fg(Color::DarkGray)
        });

        let children: Signal<Text> = children.into();
        wgt![
            style(borders(button_borders)),
            rooibos_dom::widgets::Button::new(
                children
                    .get()
                    .fg(if enabled.get() {
                        Color::Reset
                    } else {
                        Color::Gray
                    })
                    .bg(if enabled.get() {
                        Color::Reset
                    } else {
                        Color::DarkGray
                    })
            )
            .alignment(text_alignment.get())
        ]
        .layout_props(layout_props)
        .on_mouse_enter(move |_, _| border_type.set(BorderType::Double))
        .on_mouse_leave(move |_, _| border_type.set(BorderType::Round))
        .on_click({
            let on_enter = on_enter.clone();
            move |_| on_enter()
        })
        .on_focus(move |_, _| focused.set(true))
        .on_blur(move |_, _| focused.set(false))
        .on_key_down(move |props: KeyEventProps| {
            if props.event.code == KeyCode::Enter {
                on_enter()
            }
        })
        .on_key_up(key_up)
    }
}
