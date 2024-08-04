use std::error::Error;

use rooibos::dom::{col, length, row, use_window_size, widget_ref, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
use rooibos::tui::layout::Rect;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let window_size = use_window_size();
    col![
        widget_ref![props(length(2)), {
            let window_size = window_size.get();
            format!(
                "window size width={} height={}",
                window_size.width, window_size.height
            )
        }],
        row![show_size(), show_size()],
        row![show_size(), show_size()]
    ]
}

fn show_size() -> impl Render {
    let widget_size = RwSignal::new(Rect::default());
    widget_ref!({
        let widget_size = widget_size.get();
        format!(
            "x={} y={} width={} height={}",
            widget_size.x, widget_size.y, widget_size.width, widget_size.height
        )
    })
    .on_size_change(move |rect| widget_size.set(rect))
}
