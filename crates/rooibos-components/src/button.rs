use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType};
use rooibos_dom::{
    KeyCode, KeyEventProps, NodeId, WidgetState, delay, supports_keyboard_enhancement,
};
use rooibos_reactive::graph::owner::StoredValue;
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, GetValue, Set, WithValue};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;
use rooibos_reactive::{LayoutProps, Render, UpdateLayoutProps, derive_signal, wgt};
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

pub struct Button {
    on_click: Rc<RefCell<dyn FnMut()>>,
    layout_props: LayoutProps,
    border_color: MaybeSignal<Color>,
    focused_border_color: MaybeSignal<Color>,
    active_border_color: MaybeSignal<Color>,
    enabled: MaybeSignal<bool>,
    element_ref: Option<ButtonRef>,
    id: Option<NodeId>,
    class: Option<String>,
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
            enabled: true.into(),
            element_ref: None,
            id: None,
            class: None,
        }
    }

    pub fn style(mut self, props: LayoutProps) -> Self {
        self.layout_props = props;
        self
    }

    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    pub fn enabled(mut self, enabled: impl Into<MaybeSignal<bool>>) -> Self {
        self.enabled = enabled.into();
        self
    }

    pub fn element_ref(mut self, button_ref: ButtonRef) -> Self {
        self.element_ref = Some(button_ref);
        self
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
            layout_props,
            border_color,
            focused_border_color,
            active_border_color,
            enabled,
            element_ref,
            id,
            class,
        } = self;

        let border_type = RwSignal::new(BorderType::Rounded);
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
                    on_enter();
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

        let children = children.into();
        let mut button = wgt![
            rooibos_dom::Button::new(children.get())
                .block(if enabled.get() {
                    Block::bordered()
                        .bg(Color::Reset)
                        .border_type(border_type.get())
                        .border_style(Style::default().fg(current_border_color.get()))
                } else {
                    Block::bordered()
                        .bg(Color::Reset)
                        .border_set(symbols::border::QUADRANT_INSIDE)
                        .fg(Color::DarkGray)
                })
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
                .centered()
        ]
        .enabled(enabled)
        .layout_props(layout_props)
        .on_mouse_enter(move |_, _| border_type.set(BorderType::Double))
        .on_mouse_leave(move |_, _| border_type.set(BorderType::Rounded))
        .on_click({
            let on_enter = on_enter.clone();
            move |_, _, _| on_enter()
        })
        .on_focus(move |_, _| focused.set(true))
        .on_blur(move |_, _| focused.set(false))
        .on_key_down(move |props: KeyEventProps| {
            if props.event.code == KeyCode::Enter {
                on_enter()
            }
        })
        .on_key_up(key_up);
        if let Some(id) = id {
            button = button.id(id);
        }
        if let Some(class) = class {
            button = button.class(class);
        }
        button
    }
}
