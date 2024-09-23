use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::dom::{KeyCode, focus_id, text};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::layout::{align_items, chars, justify_content, position, show};
use rooibos::reactive::{
    NodeId, Render, UpdateLayoutProps, after_render, col, height, mount, wgt, width,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ExitResult, Runtime, before_exit, exit};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::{Block, Paragraph};
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

    before_exit(move || async move {
        if quit_confirmed.get() {
            return ExitResult::Exit;
        }
        set_show_popup.set(true);
        ExitResult::PreventExit
    });

    col![
        Button::new()
            .height(chars(3.))
            .width(chars(8.))
            .on_click(move || {
                exit();
            })
            .render(text!("exit")),
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
    on_confirm: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
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
            props(height!(3.), width!(40.)),
            Paragraph::new("Are you sure you want to exit? [yN]").block(Block::bordered())
        )
        .id(popup_id)
        .on_key_down(move |key_event, _, _| {
            if key_event.code == KeyCode::Char('y') {
                on_confirm();
            } else {
                on_cancel();
            }
        })
    ]
}
