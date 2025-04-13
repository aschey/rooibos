use std::process::ExitCode;

use rand::Rng;
use reqwest::Client;
use rooibos::components::Button;
use rooibos::reactive::dom::layout::padding;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, span, text};
use rooibos::reactive::graph::computed::AsyncDerived;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, error_map, suspense, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;
use serde::Deserialize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let (id, set_id) = signal(1);

    let character = AsyncDerived::new(move || fetch_next(id.get()));

    let fallback = move |errors| {
        let error_list = move || error_map(&errors, |_, e| span!(e));
        wgt!(line!(error_list()))
    };

    col![
        style(padding(1)),
        Button::new()
            .width(12)
            .on_click(move || {
                set_id.set(rand::thread_rng().gen_range(1..80));
            })
            .render(text!("fetch next")),
        suspense!(
            wgt!(" Loading...".gray()),
            character.await.map(|c| wgt!(line!(" ", c.clone().green()))),
            fallback
        )
    ]
}

#[derive(Deserialize)]
struct Response {
    result: ApiResult,
}

#[derive(Deserialize)]
struct ApiResult {
    properties: Properties,
}

#[derive(Deserialize)]
struct Properties {
    name: String,
}

async fn fetch_next(id: i32) -> rooibos::reactive::error::Result<String> {
    let res = Client::new()
        .get(format!("https://swapi.tech/api/people/{id}"))
        .send()
        .await?
        .json::<Response>()
        .await?;
    Ok(res.result.properties.name)
}
