use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use rooibos::components::Button;
use rooibos::dom::text;
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::layout::{align_items, block, chars};
use rooibos::reactive::{
    Render, UpdateLayoutProps, col, derive_signal, height, mount, row, wgt, width,
};
use rooibos::router::{Route, RouteFromStatic, Router, ToRoute, provide_router, use_router};
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
    command: CliCommands,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
    Home(Home),
    About(About),
    Blogs(BlogIndex),
    Blog(BlogPost),
}

#[derive(Route, Debug, Args)]
struct Home;

#[derive(Route, Debug, Args)]
struct About;

#[derive(Route, Debug, Args)]
struct BlogIndex;

#[derive(Route, Debug, Args)]
struct BlogPost {
    id: usize,
}

fn main() -> Result {
    let matches = Cli::parse();
    match matches.command {
        CliCommands::Home(val) => run_tui(val),
        CliCommands::About(val) => run_tui(val),
        CliCommands::Blogs(val) => run_tui(val),
        CliCommands::Blog(val) => run_tui(val),
    }
}

#[rooibos::main]
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
                    Route::new::<Home>(home),
                    Route::new::<About>(about),
                    Route::new::<BlogIndex>(blog_index),
                    Route::new::<BlogPost>(blog_post)
                ])
                .initial(initial_route)
        ],
        footer()
    ]
}

fn home() -> impl Render {
    let router = use_router();
    let about_click = move || router.push(About);
    let blog_click = move || router.push(BlogIndex);
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
    let router = use_router();
    let route_to_post = move |id: usize| router.push(BlogPost { id });
    col![
        wgt!("This is the blog page"),
        Button::new()
            .height(chars(3.))
            .on_click(move || route_to_post(1))
            .render(text!("post 1")),
        Button::new()
            .height(chars(3.))
            .on_click(move || route_to_post(2))
            .render(text!("post 2"))
    ]
}

fn blog_post() -> impl Render {
    let router = use_router();
    let id = router.use_param(BlogPost::ID);
    wgt!(format!("blog post {}", id.get()))
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
            .enabled(derive_signal!(router.can_go_back().get()))
            .render(text!("←")),
        Button::new()
            .height(chars(3.))
            .on_click(on_forward)
            .enabled(derive_signal!(router.can_go_forward().get()))
            .render(text!("→"))
    ]
}
