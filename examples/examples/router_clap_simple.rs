use std::error::Error;

use clap::Parser;
use rooibos::components::{use_router, Route, RouteFrom, Router, ToRoute};
use rooibos::dom::{after_render, col, focus_id, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::widgets::Paragraph;
use rooibos::Route;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser, Route, Debug)]
#[command(version, about)]
enum AppRoute {
    Child1,
    Child2,
}

fn main() -> Result<()> {
    let matches = AppRoute::parse();
    run_tui(matches)
}

#[rooibos::main]
async fn run_tui(route: impl ToRoute + 'static) -> Result<()> {
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default(),
        CrosstermBackend::stdout(),
        move || app(route),
    );
    runtime.run().await?;

    Ok(())
}

fn app(initial_route: impl ToRoute) -> impl Render {
    col![
        Router::new()
            .routes([
                Route::new(AppRoute::Child1, child1),
                Route::new(AppRoute::Child2, child2)
            ])
            .initial(initial_route)
    ]
}

fn child1() -> impl Render {
    let router = use_router();

    after_render(move || {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(AppRoute::Child2);
        }
    };

    widget_ref!(Paragraph::new("child1"))
        .on_key_down(key_down)
        .id("child1")
}

fn child2() -> impl Render {
    let router = use_router();

    after_render(move || {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(AppRoute::Child1);
        }
    };

    widget_ref!(Paragraph::new("child2"))
        .on_key_down(key_down)
        .id("child2")
}
