use std::error::Error;
use std::io::Stdout;

use rooibos::components::{use_router, Route, Router, Tab, TabView};
use rooibos::dom::{col, row, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::Update;
use rooibos::runtime::{run, start, RuntimeSettings, TerminalSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::text::Line;
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    start(RuntimeSettings::default(), app);
    run::<Stdout>(TerminalSettings::default()).await?;
    Ok(())
}

fn app() -> impl Render {
    col![
        Router::new()
            .initial("/tabs/tab1")
            .routes([Route::new("/tabs/{id}", tabs)])
    ]
}

fn tabs() -> impl Render {
    let router = use_router();
    let count = RwSignal::new(0);
    let current_route = router.use_param("id");

    let tabs = vec![
        Tab::new(Line::from("Tab1"), "tab1", move || "tab1"),
        Tab::new(Line::from("Tab2"), "tab2", move || "tab2"),
    ];

    row![
        TabView::new()
            .header_constraint(Length(3))
            .block(Block::bordered().title("Demo"))
            .highlight_style(Style::new().yellow())
            .fit(true)
            .on_change(move |_, tab| {
                count.update(|c| *c += 1);
                router.push(format!("/tabs/{tab}"));
            })
            .render(current_route, tabs),
    ]
}
