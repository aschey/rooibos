use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use rooibos::components::{Tab, TabList, TabView};
use rooibos::dom::{col, Constrainable, EventData, KeyCode, KeyEvent, Render};
use rooibos::reactive::owner::provide_context;
use rooibos::reactive::signal::{signal, ReadSignal, RwSignal};
use rooibos::reactive::traits::{Get, Set};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{start, RuntimeSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::text::Line;
use rooibos::tui::widgets::Block;
use tilia::transport_async::codec::{CodecStream, LengthDelimitedCodec};
use tilia::transport_async::ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use tilia::transport_async::{ipc, Bind};
use tokio::time;
use tracing::Level;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::tab0::tab0;
use crate::tab1::tab1;
use crate::tab2::tab2;

mod random;
mod tab0;
mod tab1;
mod tab2;

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

    let handle = start(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    handle.run().await?;
    guard.stop().await.unwrap();
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Tick(ReadSignal<u32>);

fn app() -> impl Render {
    let (tick, set_tick) = signal(0);
    provide_context(Tick(tick));

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(250));
        let mut seq: u32 = 1;
        loop {
            interval.tick().await;
            set_tick.set(seq);
            seq += 1;
        }
    });

    col![header_tabs()].length(3)
}

const TAB0: &str = "Tab0";
const TAB1: &str = "Tab1";
const TAB2: &str = "Tab2";

fn header_tabs() -> impl Render {
    let focused = RwSignal::new(TAB0.to_string());

    let tab_header = |title: &'static str| Line::from(title.green());

    let tabs = RwSignal::new(TabList(vec![
        Tab::new(tab_header(TAB0), TAB0.to_string(), tab0),
        Tab::new(tab_header(TAB1), TAB1.to_string(), tab1),
        Tab::new(tab_header(TAB2), TAB2.to_string(), tab2),
    ]));

    let on_key_down = move |key_event: KeyEvent, _: EventData| {
        let tabs = tabs.get();
        match key_event.code {
            KeyCode::Left => {
                if let Some(prev) = tabs.prev_tab(&focused.get()) {
                    focused.set(prev.get_value());
                }
            }
            KeyCode::Right => {
                if let Some(next) = tabs.next_tab(&focused.get()) {
                    focused.set(next.get_value());
                }
            }
            _ => {}
        }
    };

    col![
        TabView::new()
            .header_constraint(Length(3))
            .block(Block::bordered().title("Demo"))
            .highlight_style(Style::new().yellow())
            .on_key_down(on_key_down)
            .on_title_click(move |_, tab| focused.set(tab))
            .render(focused, tabs)
    ]
}
