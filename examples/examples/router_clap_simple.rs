use std::process::ExitCode;

use clap::Parser;
use rooibos::dom::{KeyCode, KeyEvent, focus_id};
use rooibos::reactive::{Render, after_render, col, mount, wgt};
use rooibos::router::{Route, RouteFrom, Router, ToRoute, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Paragraph;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[derive(Parser, Route, Debug)]
#[command(version, about)]
enum AppRoute {
    Child1,
    Child2,
}

fn main() -> Result {
    let matches = AppRoute::parse();
    run_tui(matches)
}

#[rooibos::main]
async fn run_tui(route: impl ToRoute + 'static) -> Result {
    mount(move || app(route));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
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

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(AppRoute::Child2);
        }
    };

    wgt!(Paragraph::new("child1"))
        .on_key_down(key_down)
        .id("child1")
}

fn child2() -> impl Render {
    let router = use_router();

    after_render(move || {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(AppRoute::Child1);
        }
    };

    wgt!(Paragraph::new("child2"))
        .on_key_down(key_down)
        .id("child2")
}
