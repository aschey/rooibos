use std::error::Error;

use rooibos::dom::layout::hide;
use rooibos::dom::{flex_col, flex_row, use_window_size, wgt, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set, Track, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime};
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
    let hide_row = RwSignal::new(true);
    let keypress = use_keypress();
    Effect::new(move |init: Option<()>| {
        keypress.track();
        if init.is_some() {
            hide_row.update(|h| *h = !*h);
        }
    });
    flex_col![
        wgt![{
            let window_size = window_size.get();
            format!(
                "window size width={} height={}",
                window_size.width, window_size.height
            )
        }],
        flex_row![props(hide(hide_row)), show_size(1), show_size(2)],
        flex_row![show_size(3), show_size(4)]
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
