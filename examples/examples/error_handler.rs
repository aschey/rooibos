use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::{
    Borders, align_items, borders, justify_content, position, show,
};
use rooibos::reactive::dom::{
    NodeId, Render, UpdateLayoutProps, after_render, focus_id, line, text,
};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get, Read, Set};
use rooibos::reactive::{col, derive_signal, height, max_width, padding, row, wgt, width};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ExitResult, Runtime, before_exit, exit, exit_with_error};
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::text::Line;
use taffy::{AlignItems, JustifyContent, LengthPercentage, Position};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    color_eyre::install().unwrap();
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let (popup_text, set_popup_text) = signal(None);

    before_exit(move |payload| {
        if let Some(err) = payload.error() {
            set_popup_text.set(Some(line!("An error occurred: ", err.to_string().red())));
            ExitResult::PreventExit
        } else {
            ExitResult::Exit
        }
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
                    exit_with_error("catastrophic failure!");
                })
                .render(text!("boom")),
            Button::new()
                .on_click(move || {
                    panic!("oh dear");
                })
                .render(text!("panic!")),
        ],
        // TODO: modal popup
        popup(popup_text, move || {
            exit();
        },)
    ]
}

fn popup(
    text: ReadSignal<Option<Line<'static>>>,
    on_ok: impl Fn() + Clone + Send + Sync + 'static,
) -> impl Render {
    let button_id = NodeId::new_auto();
    let show_popup = derive_signal!(text.read().is_some());

    Effect::new(move || {
        if show_popup.get() {
            after_render(move || {
                focus_id(button_id);
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
        col![
            props(borders(Borders::all()), align_items(AlignItems::Center)),
            wgt!(text.get().unwrap_or_default()),
            row![
                Button::new()
                    .centered()
                    .padding_x(LengthPercentage::Length(1.))
                    .id(button_id)
                    .on_click(move || {
                        on_ok();
                    })
                    .render(text!("Ok"))
            ]
        ]
    ]
}
