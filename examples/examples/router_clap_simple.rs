use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rooibos::components::Button;
use rooibos::dom::text;
use rooibos::reactive::layout::{align_items, block, chars};
use rooibos::reactive::{Render, UpdateLayoutProps, col, height, mount, row, wgt, width};
use rooibos::router::{Route, RouteFrom, Router, ToRoute, provide_router, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Block;
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

#[rooibos::main(flavor = "current_thread")]
async fn run_tui(initial_route: impl ToRoute + 'static) -> Result {
    mount(|| app(initial_route));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app(initial_route: impl ToRoute + 'static) -> impl Render {
    provide_router();
    col![
        props(align_items(AlignItems::Center), width!(30.),),
        col![
            props(height!(10.), block(Block::bordered())),
            Router::new()
                .routes([
                    Route::new(Routes::Home, home),
                    Route::new(Routes::About, about),
                    Route::new(Routes::Blogs, blog_index),
                ])
                .initial(initial_route)
        ],
        footer()
    ]
}

fn home() -> impl Render {
    let router = use_router();
    let about_click = move || router.push(Routes::About);
    let blog_click = move || router.push(Routes::Blogs);
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

fn footer() -> impl Render {
    let router = use_router();
    let on_forward = move || router.forward();
    let on_back = move || router.back();
    row![
        props(width!(10.), height!(3.)),
        Button::new()
            .height(chars(3.))
            .on_click(on_back)
            .enabled(router.can_go_back())
            .render(text!("←")),
        Button::new()
            .height(chars(3.))
            .on_click(on_forward)
            .enabled(router.can_go_forward())
            .render(text!("→"))
    ]
}
