use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rooibos::components::Button;
use rooibos::reactive::dom::layout::{Borders, align_items, borders, chars};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, text};
use rooibos::reactive::{col, height, row, wgt, width};
use rooibos::router::{Route, RouteContext, RouteFrom, ToRoute, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use taffy::AlignItems;

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
async fn run_tui(initial_route: impl ToRoute + 'static) -> Result {
    Runtime::initialize(DefaultBackend::auto())
        .run(|| app(initial_route))
        .await
}

fn app(initial_route: impl ToRoute + 'static) -> impl Render {
    let (router, route_context) = use_router();
    col![
        props(align_items(AlignItems::Center), width!(30.),),
        col![
            props(height!(10.), width!(100.%), borders(Borders::all())),
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
        props(align_items(AlignItems::Center)),
        wgt!(props(width!(22.), height!(2.)), "This is the home page"),
        row![
            props(width!(18.)),
            Button::new()
                .width(chars(9.))
                .height(chars(3.))
                .on_click(about_click)
                .render(text!("About")),
            Button::new()
                .width(chars(9.))
                .height(chars(3.))
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
        props(width!(10.), height!(3.)),
        Button::new()
            .height(chars(3.))
            .on_click(on_back)
            .enabled(route_context.can_go_back())
            .render(text!("←")),
        Button::new()
            .height(chars(3.))
            .on_click(on_forward)
            .enabled(route_context.can_go_forward())
            .render(text!("→"))
    ]
}
