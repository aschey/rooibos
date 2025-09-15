use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::layout::Alignment;
use ratatui::text::Text;
use rooibos_dom::events::KeyEventProps;
use rooibos_dom::{KeyCode, supports_key_up};
use rooibos_reactive::dom::layout::{Borders, borders};
use rooibos_reactive::dom::{LayoutProps, Render, UpdateLayoutProps};
use rooibos_reactive::graph::owner::StoredValue;
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, GetValue, Set, WithValue};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_reactive::{StateProp, delay, derive_signal, use_state_prop, wgt};
use tokio::sync::broadcast;
use tokio::task::spawn_local;
use tui_theme::Style;

use crate::{ColorThemeColorTheme, with_theme};

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
    button_style: Signal<StateProp<Style>>,
    active_button_style: Signal<Style>,
    button_borders: Signal<StateProp<Borders>>,
    active_button_borders: Signal<Borders>,
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
            button_style: with_theme(|_| {
                StateProp::new(Style::default())
                    .disabled(|s| s.fg_disabled_light().bg_disabled_dark())
            }),
            active_button_style: Style::new().into(),
            button_borders: with_theme(|theme| {
                let props = theme.app_properties;
                StateProp::new(
                    Borders::all()
                        .border_type(props.border_type_primary)
                        .fg_border(),
                )
                .focused(|b| b.fg_border_focused())
                .hovered(move |b| b.border_type(props.border_type_hovered))
                .disabled(move |b| b.border_type(props.border_type_disabled).fg_disabled_dark())
            }),
            active_button_borders: with_theme(|theme| {
                let props = theme.app_properties;
                Borders::all()
                    .border_type(props.border_type_active)
                    .fg_active()
            }),
            text_alignment: Alignment::Left.into(),
            element_ref: None,
        }
    }

    pub fn element_ref(mut self, button_ref: ButtonRef) -> Self {
        self.element_ref = Some(button_ref);
        self
    }

    pub fn button_style(mut self, button_style: impl Into<Signal<StateProp<Style>>>) -> Self {
        self.button_style = button_style.into();
        self
    }

    pub fn active_button_style(mut self, active_button_style: impl Into<Signal<Style>>) -> Self {
        self.active_button_style = active_button_style.into();
        self
    }

    pub fn borders(mut self, borders: impl Into<Signal<StateProp<Borders>>>) -> Self {
        self.button_borders = borders.into();
        self
    }

    pub fn active_borders(mut self, active_borders: impl Into<Signal<Borders>>) -> Self {
        self.active_button_borders = active_borders.into();
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
            button_borders,
            active_button_borders,
            button_style,
            active_button_style,
            element_ref,
            text_alignment,
        } = self;
        let enabled = layout_props.enabled.value().unwrap_or(true.into());

        let active = RwSignal::new(false);

        let (button_style, set_button_state) = use_state_prop(button_style);
        let (button_borders, set_border_state) = use_state_prop(button_borders);

        let current_button_style = derive_signal!({
            if active.get() {
                active_button_style.get()
            } else {
                button_style.get()
            }
        });

        let on_enter = {
            let on_click = on_click.clone();
            move || {
                active.set(true);
                if !supports_key_up() {
                    delay(Duration::from_millis(50), async move {
                        // Need to use try_set here in case the button was already disposed
                        active.try_set(false);
                    });
                }
                on_click.borrow_mut()()
            }
        };

        // TODO: support mouse up events and combine this with on_enter
        let on_mouse_down = move || {
            active.set(true);
            delay(Duration::from_millis(50), async move {
                // Need to use try_set here in case the button was already disposed
                active.try_set(false);
            });

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

        let button_borders = derive_signal!(if active.get() {
            active_button_borders.get()
        } else {
            button_borders.get()
        });

        let children: Signal<Text> = children.into();
        wgt![
            style(borders(button_borders)),
            rooibos_dom::widgets::Button::new(children.get().style(current_button_style.get()))
                .alignment(text_alignment.get())
        ]
        .layout_props(layout_props)
        .on_click(move |_| on_mouse_down())
        .on_state_change(set_button_state)
        .on_state_change(set_border_state)
        .on_key_down(move |props: KeyEventProps| {
            if props.event.code == KeyCode::Enter {
                on_enter()
            }
        })
        .on_key_up(move |_| {
            active.set(false);
        })
    }
}
