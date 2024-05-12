use std::error::Error;
use std::io::Stdout;

use rand::Rng;
use reqwest::Client;
use rooibos::components::{container, Button};
use rooibos::dom::{col, signal, transition, Render, Suspend};
use rooibos::reactive::actions::Action;
use rooibos::reactive::computed::AsyncDerived;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::reactive::wrappers::read::Signal;
use rooibos::runtime::{run, start, RuntimeSettings, TerminalSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::text::Text;
use serde::Deserialize;
use tilia::transport_async::codec::{CodecStream, LengthDelimitedCodec};
use tilia::transport_async::ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use tilia::transport_async::{ipc, Bind};
use tracing::Level;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let (ipc_writer, mut guard) = tilia::Writer::new(1024, move || {
        Box::pin(async move {
            let transport = ipc::Endpoint::bind(
                ipc::EndpointParams::new(
                    ServerId("rooibos-demo"),
                    SecurityAttributes::allow_everyone_create().unwrap(),
                    OnConflict::Overwrite,
                )
                .unwrap(),
            )
            .await
            .unwrap();
            CodecStream::new(transport, LengthDelimitedCodec)
        })
    });

    tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                .add_directive(Level::TRACE.into())
                .add_directive("tokio_util=info".parse().unwrap())
                .add_directive("tokio_tower=info".parse().unwrap()),
        )
        .with({
            Layer::new()
                .compact()
                .with_writer(ipc_writer)
                .with_filter(tilia::Filter::default())
        })
        .init();

    start(RuntimeSettings::default(), app);
    run::<Stdout>(TerminalSettings::default()).await?;
    guard.stop().await.unwrap();
    Ok(())
}

fn app() -> impl Render {
    let (id, set_id) = signal(1);
    let character = AsyncDerived::new(move || fetch_next(id.get()));

    col![
        Button::new()
            .on_click(move || {
                set_id.set(rand::thread_rng().gen_range(1..80));
            })
            .render(Text::from("fetch next")),
        transition(
            || "Loading...",
            move || Suspend(async move { character.await }),
        )
    ]
}

#[derive(Deserialize)]
struct Response {
    name: String,
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
    res.name
}
