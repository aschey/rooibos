use std::process::ExitCode;

use rooibos::reactive::dom::layout::{Borders, borders, full, half, height, width};
use rooibos::reactive::dom::{Render, span};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, max_viewport_width, use_window_focus, use_window_size};
use rooibos::terminal::DefaultBackend;
use rooibos::tui::layout::Rect;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(|_| app())
        .await
}

fn app() -> impl Render {
    max_viewport_width(100).unwrap();

    let window_size = use_window_size();
    let window_focused = use_window_focus();

    col![
        style(height(full()), width(full())),
        wgt!(style(borders(Borders::all())), {
            let window_size = window_size.get().viewport();
            span!(
                "window size width={} height={} focused={}",
                window_size.width,
                window_size.height,
                window_focused.get()
            )
        }),
        row![
            style(height(half()), width(full())),
            show_size(1),
            show_size(2)
        ],
        row![
            style(height(half()), width(full())),
            show_size(3),
            show_size(4)
        ]
    ]
}

fn show_size(id: usize) -> impl Render {
    let widget_size = RwSignal::new(Rect::default());
    wgt!(style(width(full()), borders(Borders::all())), {
        let widget_size = widget_size.get();
        span!(
            "id:{id} x={} y={} width={} height={}",
            widget_size.x,
            widget_size.y,
            widget_size.width,
            widget_size.height
        )
    })
    .on_size_change(move |rect| widget_size.set(rect))
}
