use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::keybind::{Bind, keys, map_handler};
use rooibos::reactive::dom::layout::{
    Borders, align_items, borders, justify_content, position, show,
};
use rooibos::reactive::dom::{
    NodeId, Render, after_render, focus_id, text,
};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, height, max_width, padding, row, wgt, width};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ExitResult, Runtime, before_exit, exit};
use rooibos::terminal::crossterm::CrosstermBackend;
use taffy::{AlignItems, JustifyContent, Position};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(CrosstermBackend::stdout())
        .run(app)
        .await
}

fn app() -> impl Render {
    let (show_popup, set_show_popup) = signal(false);
    let (quit_confirmed, set_quit_confirmed) = signal(false);

    before_exit(move |payload| async move {
        // We should always exit when we have a non-success code
        if !payload.is_success() || quit_confirmed.get() {
            return ExitResult::Exit;
        }
        set_show_popup.set(true);
        ExitResult::PreventExit
    });

    col![
        props(
            padding!(1.),
            width!(100.%),
            height!(100.%),
            max_width!(100.),
            borders(Borders::all())
        ),
        row![
            Button::new()
                .on_click(move || {
                    exit();
                })
                .render(text!("exit")),
        ],
        // TODO: modal popup
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
