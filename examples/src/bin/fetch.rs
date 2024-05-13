use std::error::Error;
use std::io::Stdout;

use rand::Rng;
use reqwest::Client;
use rooibos::components::Button;
use rooibos::dom::{col, row, suspense, widget_ref, Constrainable, Render, Suspend};
use rooibos::reactive::computed::AsyncDerived;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Set};
use rooibos::runtime::{run, start, RuntimeSettings, TerminalSettings};
use rooibos::tui::style::Stylize;
use rooibos::tui::text::{Line, Text};
use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    start(RuntimeSettings::default(), app);
    run::<Stdout>(TerminalSettings::default()).await?;

    Ok(())
}

fn app() -> impl Render {
    let (id, set_id) = signal(1);

    let character = AsyncDerived::new(move || fetch_next(id.get()));

    col![
        row![
            col![
                Button::new()
                    .on_click(move || {
                        set_id.set(rand::thread_rng().gen_range(1..80));
                    })
                    .length(3)
                    .render(Text::from("fetch next")),
            ]
            .length(20)
        ]
        .length(3),
        row![col![suspense(
            move || widget_ref!(Line::from("Loading...".gray())),
            move || {
                Suspend(async move {
                    let character = character.await;
                    widget_ref!(Line::from(character.clone().green()))
                })
            }
        )]]
        .length(3)
    ]
}

#[derive(Deserialize)]
struct Response {
    name: Option<String>,
}

async fn fetch_next(id: i32) -> String {
    let res = Client::new()
        .get(format!("https://swapi.dev/api/people/{id}"))
        .send()
        .await
        .unwrap()
        .json::<Response>()
        .await
        .unwrap();
    res.name.unwrap_or_else(|| "N/A".to_string())
}
