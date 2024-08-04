use std::error::Error;

use clap::{Args, Parser, Subcommand};
use rooibos::components::{use_router, DefaultRoute, Route, RouteFromStatic, Router, ToRoute};
use rooibos::dom::{after_render, col, focus_id, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::widgets::Paragraph;
use rooibos::Route;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand, Debug)]
#[command(version, about)]
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

fn main() -> Result<()> {
    let matches = Cli::parse();
    match matches.command {
        CliCommands::Cmd1(cmd1) => run_tui(cmd1),
        CliCommands::Cmd2(cmd2) => run_tui(cmd2),
        res => {
            println!("{res:?}");
            Ok(())
        }
    }
}

#[rooibos::main]
async fn run_tui(route: impl ToRoute + 'static) -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::stdout(),
        move || app(route),
    );
    runtime.run().await?;

    Ok(())
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

    Effect::new(move |_| {
        focus_id("child0");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Cmd1::new(Some(1)));
        }
    };

    widget_ref!(Paragraph::new("child0"))
        .on_key_down(key_down)
        .id("child0")
}

fn child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.try_use_query(Cmd1::ID);

    after_render(move || {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Cmd2::new(child2_id.get()));
            child2_id.update(|id| *id += 1);
        }
    };

    widget_ref!(Paragraph::new(format!(
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

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Cmd1::new(Some(1)));
        }
    };

    widget_ref!(Paragraph::new(format!("child2 id={}", id.get())))
        .on_key_down(key_down)
        .id("child2")
}
