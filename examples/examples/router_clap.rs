use std::error::Error;
use std::io::Stdout;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use rooibos::components::{use_router, Route, Router};
use rooibos::dom::{after_render, col, focus_id, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::widgets::Paragraph;
use rooibos::Routes;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

static CHILD1: &str = "child1";
static CHILD2: &str = "child2";

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Parser, Debug, Routes)]
#[command(version, about)]
enum CliCommands {
    #[route(CHILD1)]
    Cmd1 {
        #[arg(short, long)]
        id: Option<i32>,
    },
    #[route(CHILD2)]
    Cmd2 {
        #[arg(short, long)]
        id: i32,
    },
    Cmd3,
}

fn main() -> Result<()> {
    let command = Cli::command();
    let (matches, route) = CliCommands::create_route_from(command);

    if let Some(route) = route {
        run_tui(route)
    } else {
        let cmd = CliCommands::from_arg_matches(&matches).unwrap();
        println!("{cmd:?}");
        Ok(())
    }
}

#[rooibos::main]
async fn run_tui(route: String) -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        move || app(route),
    );
    runtime.run().await?;

    Ok(())
}

fn app(initial_route: String) -> impl Render {
    let child2_id = RwSignal::new(0);
    col![
        Router::new()
            .routes([
                Route::new("/", child0),
                Route::new(format!("/{CHILD1}"), move || child1(child2_id)),
                Route::new(format!("/{CHILD2}/{{id}}"), child2)
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
            router.push(format!("/{CHILD1}?id=1"));
        }
    };

    widget_ref!(Paragraph::new("child0"))
        .on_key_down(key_down)
        .id("child0")
}

fn child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.try_use_query("id");

    after_render(move || {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(format!("/{CHILD2}/{}", child2_id.get()));
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
    let id = router.use_param("id");

    after_render(move || {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(format!("/{CHILD1}?id=1"));
        }
    };

    widget_ref!(Paragraph::new(format!("child2 id={}", id.get())))
        .on_key_down(key_down)
        .id("child2")
}
