use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use rooibos::dom::{KeyCode, KeyEvent, focus_id};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, after_render, col, mount, wgt};
use rooibos::router::{DefaultRoute, Route, RouteFromStatic, Router, ToRoute, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Paragraph;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
    #[command(name = "child1")]
    Cmd1(Cmd1),
    #[command(name = "child2")]
    Cmd2(Cmd2),
    Cmd3,
}

#[derive(Args, Debug, Route)]
struct Cmd1 {
    #[arg(short, long)]
    id: Option<i32>,
}

impl Cmd1 {
    fn new(id: Option<i32>) -> Self {
        Self { id }
    }
}

#[derive(Args, Debug, Route)]
struct Cmd2 {
    #[arg(short, long)]
    id: i32,
}

impl Cmd2 {
    fn new(id: i32) -> Self {
        Self { id }
    }
}

fn main() -> Result {
    let matches = Cli::parse();
    match matches.command {
        CliCommands::Cmd1(cmd1) => run_tui(cmd1),
        CliCommands::Cmd2(cmd2) => run_tui(cmd2),
        res => {
            println!("{res:?}");
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[rooibos::main]
async fn run_tui(route: impl ToRoute + 'static) -> Result {
    mount(move || app(route));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app(initial_route: impl ToRoute) -> impl Render {
    let child2_id = RwSignal::new(0);

    col![
        Router::new()
            .routes([
                Route::new::<DefaultRoute>(child0),
                Route::new::<Cmd1>(move || child1(child2_id)),
                Route::new::<Cmd2>(child2)
            ])
            .initial(initial_route)
    ]
}

fn child0() -> impl Render {
    let router = use_router();

    Effect::new(move || {
        focus_id("child0");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Cmd1::new(Some(1)));
        }
    };

    wgt!(Paragraph::new("child0"))
        .on_key_down(key_down)
        .id("child0")
}

fn child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.try_use_query(Cmd1::ID);

    after_render(move || {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Cmd2::new(child2_id.get()));
            child2_id.update(|id| *id += 1);
        }
    };

    wgt!(Paragraph::new(format!(
        "child1 id={}",
        id.get().unwrap_or_else(|| "N/A".to_string())
    )))
    .on_key_down(key_down)
    .id("child1")
}

fn child2() -> impl Render {
    let router = use_router();
    let id = router.use_param(Cmd2::ID);

    after_render(move || {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Cmd1::new(Some(1)));
        }
    };

    wgt!(Paragraph::new(format!("child2 id={}", id.get())))
        .on_key_down(key_down)
        .id("child2")
}
