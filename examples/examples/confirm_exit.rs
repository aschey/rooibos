use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::keybind::{Bind, keys, map_handler};
use rooibos::reactive::dom::layout::{
    Borders, align_items, borders, chars, justify_content, position, show,
};
use rooibos::reactive::dom::{
    NodeId, Render, UpdateLayoutProps, after_render, focus_id, mount, text,
};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, height, row, wgt, width};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ExitResult, Runtime, before_exit, exit, signal};
use rooibos::terminal::crossterm::CrosstermBackend;
use taffy::{AlignItems, JustifyContent, Position};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let (show_popup, set_show_popup) = signal(false);
    let (quit_confirmed, set_quit_confirmed) = signal(false);

    before_exit(move |payload| async move {
        // We should always exit when we receive a termination signal
        if payload.signal().is_some() || quit_confirmed.get() {
            return ExitResult::Exit;
        }
        set_show_popup.set(true);
        ExitResult::PreventExit
    });

    col![
        row![
            Button::new()
                .on_click(move || {
                    exit();
                })
                .render(text!("exit")),
        ],
        popup(
            show_popup,
            move || {
                set_quit_confirmed.set(true);
                exit();
            },
            move || {
                set_show_popup.set(false);
            }
        )
    ]
}

fn popup(
    show_popup: ReadSignal<bool>,
    on_confirm: impl Fn() + Send + Sync + 'static,
    on_cancel: impl Fn() + Send + Sync + 'static,
) -> impl Render {
    let popup_id = NodeId::new_auto();

    Effect::new(move || {
        if show_popup.get() {
            after_render(move || {
                focus_id(popup_id);
            });
        };
    });

    col![
        props(
            width!(100.%),
            height!(100.%),
            show(show_popup),
            position(Position::Absolute),
            align_items(AlignItems::Center),
            justify_content(JustifyContent::Center),
        ),
        wgt!(
            props(borders(Borders::all())),
            "Are you sure you want to exit? [yN]"
        )
        .id(popup_id)
        .on_key_down(
            [
                map_handler("y", move |_, _| {
                    on_confirm();
                }),
                map_handler(keys::ANY, move |_, _| {
                    on_cancel();
                })
            ]
            .bind()
        )
    ]
}
