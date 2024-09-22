use std::error::Error;

use rand::Rng;
use reqwest::Client;
use rooibos::components::Button;
use rooibos::dom::{line, span, text};
use rooibos::reactive::graph::computed::AsyncDerived;
use rooibos::reactive::graph::signal::{ArcRwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set, With};
use rooibos::reactive::layout::chars;
use rooibos::reactive::{
    Errors, Render, UpdateLayoutProps, col, max_width, mount, padding, suspense, wgt,
};
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::Paragraph;
use serde::Deserialize;

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;

    Ok(())
}

fn app() -> impl Render {
    let (id, set_id) = signal(1);

    let character = AsyncDerived::new(move || fetch_next(id.get()));

    let fallback = move |errors: ArcRwSignal<Errors>| {
        let error_list =
            move || errors.with(|errors| errors.iter().map(|(_, e)| span!(e)).collect::<Vec<_>>());

        wgt!(Paragraph::new(line!(error_list())))
    };

    col![
        props(max_width!(25.), padding!(1.)),
        Button::new()
            .on_click(move || {
                set_id.set(rand::thread_rng().gen_range(1..80));
            })
            .height(chars(3.))
            .render(text!("fetch next")),
        suspense!(
            wgt!(line!(" Loading...".gray())),
            character.await.map(|c| wgt!(line!(" ", c.clone().green()))),
            fallback
        )
    ]
}

#[derive(Deserialize)]
struct Response {
    name: String,
}

async fn fetch_next(id: i32) -> rooibos::reactive::Result<String> {
    let res = Client::new()
        .get(format!("https://swapi.dev/api/people/{id}"))
        .send()
        .await?
        .json::<Response>()
        .await?;
    Ok(res.name)
}
