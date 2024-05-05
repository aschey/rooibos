use std::error::Error;
use std::time::Duration;

use crossterm::event::KeyCode;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::owner::provide_context;
use rooibos::reactive::signal::{signal, ReadSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::reactive::wrappers::read::Signal;
use rooibos::runtime::{run, use_keypress};
use tilia::transport_async::codec::{CodecStream, LengthDelimitedCodec};
use tilia::transport_async::ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use tilia::transport_async::{ipc, Bind};
use tokio::time;
use tracing::Level;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::tab0::{tab_0, Tab0Props};
use crate::tab1::{tab_1, Tab1Props};
use crate::tab2::{tab_2, Tab2Props};

mod random;
mod tab0;
mod tab1;
mod tab2;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const NUM_TABS: usize = 3;

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

    mount(|| view!(<App/>));
    run().await?;
    guard.stop().await.unwrap();
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Tick(ReadSignal<u32>);

#[component]
fn App() -> impl Render {
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

    view! {
        <Col v:length=3>
            <HeaderTabs/>
        </Col>

    }
}

const TAB0: &str = "Tab0";
const TAB1: &str = "Tab1";
const TAB2: &str = "Tab2";

#[component]
fn HeaderTabs() -> impl Render {
    let (focused_tab, set_focused_tab) = signal(0);

    let titles = [TAB0, TAB1, TAB2];
    let focused_title = Signal::derive(move || titles[focused_tab.get()].to_string());

    let update_current_tab = move |change: i32| {
        set_focused_tab.update(|f| {
            let next = (*f as i32 + change).rem_euclid(NUM_TABS as i32);
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

    let tab_header = |title: &'static str| prop!(<Line><Span green>{title}</Span></Line>);

    view! {
        <Col>
            <Tabs
                padding=1
                block=prop!(<Block borders=Borders::ALL title="Demo"/>)
                highlight_style=prop!(<Style yellow/>)
                current_tab=focused_title
            >
                <Tab
                    header=tab_header(TAB0)
                    value = TAB0.to_string()
                >
                    {move || any_view!(<Tab0/>)}
                </Tab>
                <Tab
                    header=tab_header(TAB1)
                    value = TAB1.to_string()
                >
                    {move || any_view!(<Tab1/>)}
                </Tab>
                <Tab
                    header=tab_header(TAB2)
                    value = TAB2.to_string()
                >
                    {move || any_view!(<Tab2/>)}
                </Tab>
            </Tabs>

        </Col>
    }
}
