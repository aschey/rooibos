use std::time::Duration;

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Set};
use rooibos_dom::prelude::*;
use rooibos_runtime::{delay, supports_key_up};

#[component]
pub fn Button<F, M, C>(#[prop(children)] children: F, mut on_click: C) -> impl Render
where
    C: FnMut() + 'static,
    F: Fn() -> M + 'static,
    M: Into<Text<'static>> + 'static,
{
    let border_color = RwSignal::new(Color::Gray);

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Enter {
            border_color.set(Color::Green);

            if !supports_key_up() {
                delay(Duration::from_millis(50), async move {
                    border_color.set(Color::Blue);
                });
            }

            on_click()
        }
    };

    let key_up = move |key_event: KeyEvent| {
        if !supports_key_up() {
            return;
        }
        if key_event.code == KeyCode::Enter {
            border_color.set(Color::Blue);
        }
    };
    view! {
        <paragraph
            v:focusable
            on:focus=move || {
                border_color.set(Color::Blue);
            }
            on:blur=move || {
                border_color.set(Color::Gray);
            }
            on:key_down=key_down
            on:key_up=key_up
            block=prop! {
                <Block
                    borders=Borders::ALL
                    border_type=BorderType::Rounded
                    border_style=prop! {
                        <Style
                            fg=border_color.get()
                        /> }
                    />
                }
            alignment=Alignment::Center
        >
            {children()}
        </paragraph>
    }
}
