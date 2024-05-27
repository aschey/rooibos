use std::error::Error;
use std::io::Stdout;

use rooibos::components::{use_router, Button, KeyedWrappingList, Route, Router, Tab, TabView};
use rooibos::dom::{col, row, Constrainable, EventData, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::text::{Line, Text};
use rooibos::tui::widgets::Block;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    runtime.run().await?;
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

    let tabs = KeyedWrappingList(vec![
        Tab::new(Line::from("Tab1"), "tab1", move || "tab1"),
        Tab::new(Line::from("Tab2"), "tab2", move || "tab2"),
        Tab::new(Line::from("Tab3"), "tab3", move || "tab3"),
    ]);

    let on_key_down = {
        let tabs = tabs.clone();
        move |key_event: KeyEvent, _: EventData| match key_event.code {
            KeyCode::Left => {
                if let Some(prev) = tabs.prev_item(&current_route.get()) {
                    router.push(format!("/tabs/{}", prev.get_value()));
                }
            }
            KeyCode::Right => {
                if let Some(next) = tabs.next_item(&current_route.get()) {
                    router.push(format!("/tabs/{}", next.get_value()));
                }
            }
            _ => {}
        }
    };

    row![
        TabView::new()
            .header_constraint(Length(3))
            .block(Block::bordered().title("Demo"))
            .highlight_style(Style::new().yellow())
            .fit(true)
            .on_title_click(move |_, tab| {
                count.update(|c| *c += 1);
                router.push(format!("/tabs/{tab}"));
            })
            .on_key_down(on_key_down)
            .render(current_route, tabs),
        col![
            Button::new()
                .length(3)
                .on_click(move || {
                    router.pop();
                })
                .render(Text::from("Previous"))
        ]
        .length(10)
    ]
}
