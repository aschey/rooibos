use std::process::ExitCode;

use ratatui::symbols::merge::MergeStrategy;
use rooibos::reactive::dom::Render;
use rooibos::reactive::dom::layout::{Borders, borders, full, height, pct, width};
use rooibos::reactive::{col, row};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(|_| app())
        .await
}

fn app() -> impl Render {
    // TODO actually collapse the borders using grid layout after that's implemented
    row![
        style(width(full()), height(full())),
        col![style(
            width(pct(50)),
            borders(Borders::all().merge(MergeStrategy::Exact))
        )],
        col![
            style(width(pct(50))),
            row!(style(
                height(pct(50)),
                borders(Borders::all().merge(MergeStrategy::Exact))
            )),
            row!(style(
                height(pct(50)),
                borders(Borders::all().merge(MergeStrategy::Exact))
            ))
        ]
    ]
}
