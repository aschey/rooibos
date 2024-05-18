use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use rooibos::components::{Tab, TabView};
use rooibos::dom::{col, derive_signal, Constrainable, KeyCode, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::owner::provide_context;
use rooibos::reactive::signal::{signal, ReadSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::{run, start, use_keypress, RuntimeSettings, TerminalSettings};
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

    start(RuntimeSettings::default(), app);
    run::<Stdout>(TerminalSettings::default()).await?;
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
    let (focused_tab, set_focused_tab) = signal(0);

    let titles = [TAB0, TAB1, TAB2];
    let focused_title = derive_signal!(titles[focused_tab.get()].to_string());

    let update_current_tab = move |change: i32| {
        set_focused_tab.update(|f| {
            let next = (*f as i32 + change).rem_euclid(titles.len() as i32);
            *f = next as usize;
        });
    };

    let previous_tab = move || update_current_tab(-1);
    let next_tab = move || update_current_tab(1);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            match term_signal.code {
                KeyCode::Left => {
                    previous_tab();
                }
                KeyCode::Right => {
                    next_tab();
                }
                _ => {}
            }
        }
    });

    let tab_header = |title: &'static str| Line::from(title.green());

    col![
        TabView::new()
            .header_constraint(Length(3))
            .block(Block::bordered().title("Demo"))
            .highlight_style(Style::new().yellow())
            .on_change(move |i, _| set_focused_tab.set(i))
            .render(
                focused_title,
                vec![
                    Tab::new(tab_header(TAB0), TAB0.to_string(), tab0),
                    Tab::new(tab_header(TAB1), TAB1.to_string(), tab1),
                    Tab::new(tab_header(TAB2), TAB2.to_string(), tab2)
                ]
            )
    ]
}
