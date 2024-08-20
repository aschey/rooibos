use std::error::Error;

use rand::Rng;
use reqwest::Client;
use rooibos::components::Button;
use rooibos::dom::layout::chars;
use rooibos::dom::{
    col, line, max_width, span, suspense, text, wgt, Errors, Render, UpdateLayoutProps,
};
use rooibos::reactive::computed::AsyncDerived;
use rooibos::reactive::signal::{signal, ArcRwSignal};
use rooibos::reactive::traits::{Get, Set, With};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::Paragraph;
use serde::Deserialize;

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
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
        props(max_width!(25.)),
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

async fn fetch_next(id: i32) -> rooibos::dom::Result<String> {
    let res = Client::new()
        .get(format!("https://swapi.dev/api/people/{id}"))
        .send()
        .await?
        .json::<Response>()
        .await?;
    Ok(res.name)
}
