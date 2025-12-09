use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::{
    Borders, absolute, align_items, borders, center, full, height, justify_content, padding,
    position, show, width,
};
use rooibos::reactive::dom::{
    NodeId, Render, UpdateLayoutProps, after_render, focus_id, line, text,
};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get, Read, Set};
use rooibos::reactive::{IntoSignal, col, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{
    ControlFlow, Runtime, before_exit, exit, exit_with_error, max_viewport_width,
};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;
use rooibos::tui::text::Line;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    color_eyre::install().unwrap();
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(app)
        .await
}

fn app() -> impl Render {
    max_viewport_width(100).unwrap();

    let (popup_text, set_popup_text) = signal(None);

    before_exit(move |payload| {
        if let Some(err) = payload.error() {
            set_popup_text.set(Some(line!("An error occurred: ", err.to_string().red())));
            ControlFlow::Prevent
        } else {
            ControlFlow::Allow
        }
    });

    col![
        style(
            padding(1),
            width(full()),
            height(full()),
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
    let show_popup = (move || text.read().is_some()).signal();

    Effect::new(move || {
        if show_popup.get() {
            after_render(move || {
                focus_id(button_id);
            });
        };
    });

    col![
        style(
            width(full()),
            height(full()),
            show(show_popup),
            position(absolute()),
            align_items(center()),
            justify_content(center()),
        ),
        col![
            style(borders(Borders::all()), align_items(center())),
            wgt!(text.get().unwrap_or_default()),
            row![
                Button::new()
                    .centered()
                    .padding_x(1)
                    .id(button_id)
                    .on_click(move || {
                        on_ok();
                    })
                    .render(text!("Ok"))
            ]
        ]
    ]
}
