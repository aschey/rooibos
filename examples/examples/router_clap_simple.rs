use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rooibos::components::Button;
use rooibos::reactive::dom::layout::{Borders, align_items, borders, center, full, height, width};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, text};
use rooibos::reactive::{col, row, wgt};
use rooibos::router::{Route, RouteContext, RouteFrom, ToRoute, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Routes,
}

#[derive(Subcommand, Route, Debug)]
enum Routes {
    Home,
    About,
    Blogs,
}

fn main() -> Result {
    let matches = Cli::parse();
    run_tui(matches.command)
}

#[rooibos::main]
async fn run_tui(initial_route: impl ToRoute + Send + Sync + 'static) -> Result {
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(|_| app(initial_route))
        .await
}

fn app(initial_route: impl ToRoute + 'static) -> impl Render {
    let (router, route_context) = use_router();
    col![
        style(align_items(center()), width(30)),
        col![
            style(height(10), width(full()), borders(Borders::all())),
            router
                .routes([
                    Route::new(Routes::Home, move || home(route_context)),
                    Route::new(Routes::About, about),
                    Route::new(Routes::Blogs, blog_index),
                ])
                .initial(initial_route)
                .render()
        ],
        footer(route_context)
    ]
}

fn home(route_context: RouteContext) -> impl Render {
    let about_click = move || route_context.push(Routes::About);
    let blog_click = move || route_context.push(Routes::Blogs);
    col![
        style(align_items(center())),
        wgt!(style(width(22), height(2)), "This is the home page"),
        row![
            style(width(18)),
            Button::new()
                .width(9)
                .height(3)
                .on_click(about_click)
                .render(text!("About")),
            Button::new()
                .width(9)
                .height(3)
                .on_click(blog_click)
                .render(text!("Blog"))
        ]
    ]
}

fn about() -> impl Render {
    wgt!("This is the about page")
}

fn blog_index() -> impl Render {
    wgt!("This is the blog page")
}

fn footer(route_context: RouteContext) -> impl Render {
    let on_forward = move || route_context.forward();
    let on_back = move || route_context.back();
    row![
        style(width(10), height(3)),
        Button::new()
            .height(3)
            .on_click(on_back)
            .enabled(route_context.can_go_back())
            .render(text!("←")),
        Button::new()
            .height(3)
            .on_click(on_forward)
            .enabled(route_context.can_go_forward())
            .render(text!("→"))
    ]
}
