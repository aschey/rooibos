use std::process::ExitCode;

use rooibos::reactive::dom::{Render, mount, use_window_focus, use_window_size};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::layout::Rect;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());

    runtime.run().await
}

fn app() -> impl Render {
    let window_size = use_window_size();
    let window_focused = use_window_focus();

    col![
        wgt![{
            let window_size = window_size.get();
            format!(
                "window size width={} height={} focused={}",
                window_size.width,
                window_size.height,
                window_focused.get()
            )
        }],
        row![show_size(1), show_size(2)],
        row![show_size(3), show_size(4)]
    ]
}

fn show_size(id: usize) -> impl Render {
    let widget_size = RwSignal::new(Rect::default());
    wgt!({
        let widget_size = widget_size.get();
        format!(
            "id:{id} x={} y={} width={} height={}",
            widget_size.x, widget_size.y, widget_size.width, widget_size.height
        )
    })
    .on_size_change(move |rect| widget_size.set(rect))
}
