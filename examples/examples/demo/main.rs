use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::{KeyedWrappingList, Tab, TabView};
use rooibos::dom::{KeyCode, KeyEvent, line};
use rooibos::reactive::graph::owner::provide_context;
use rooibos::reactive::graph::signal::{ReadSignal, RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::layout::{chars, pct};
use rooibos::reactive::{Render, mount};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::Block;
use tokio::time;

use crate::tab0::tab0;
use crate::tab1::tab1;
use crate::tab2::tab2;

mod random;
mod tab0;
mod tab1;
mod tab2;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
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

    header_tabs()
}

const TAB0: &str = "Tab0";
const TAB1: &str = "Tab1";
const TAB2: &str = "Tab2";

fn header_tabs() -> impl Render {
    let focused = RwSignal::new(TAB0.to_string());

    let tab_header = |title: &'static str| line!(title.green());

    let tabs = RwSignal::new(KeyedWrappingList(vec![
        Tab::new(tab_header(TAB0), TAB0.to_string(), tab0),
        Tab::new(tab_header(TAB1), TAB1.to_string(), tab1),
        Tab::new(tab_header(TAB2), TAB2.to_string(), tab2),
    ]));

    let on_key_down = move |key_event: KeyEvent, _, _| {
        let tabs = tabs.get();
        match key_event.code {
            KeyCode::Left => {
                if let Some(prev) = tabs.prev_item(&focused.get()) {
                    focused.set(prev.get_value().to_string());
                }
            }
            KeyCode::Right => {
                if let Some(next) = tabs.next_item(&focused.get()) {
                    focused.set(next.get_value().to_string());
                }
            }
            _ => {}
        }
    };

    TabView::new()
        .header_height(chars(3.))
        .body_height(pct(100.))
        .block(Block::bordered().title("Demo"))
        .highlight_style(Style::new().yellow())
        .on_key_down(on_key_down)
        .on_title_click(move |_, tab| focused.set(tab.to_string()))
        .render(focused, tabs)
}
