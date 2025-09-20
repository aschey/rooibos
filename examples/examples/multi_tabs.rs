use std::process::ExitCode;

use rooibos::components::{KeyedWrappingList, Tab, TabView};
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::events::KeyEventProps;
use rooibos::reactive::dom::layout::{Borders, borders, max_height, max_width};
use rooibos::reactive::dom::{Render, line};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{KeyCode, col, row};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::{Style, Stylize};
use rooibos::tui::widgets::Block;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?).run(app).await
}

fn app() -> impl Render {
    let focused = RwSignal::new("tab1".to_string());
    let tab_block = RwSignal::new(Block::bordered().title("Demo"));

    let tabs = RwSignal::new(KeyedWrappingList::<Tab>(vec![
        Tab::new(line!("Tab1"), "tab1", move || "tab1"),
        Tab::new(line!("Tab2"), "tab2", inner_tabs),
        Tab::new(line!("Tab3"), "tab3", move || "tab3"),
    ]));

    col![
        style(max_width(50), max_height(20), borders(Borders::all())),
        TabView::new()
            .header_height(3)
            .block(tab_block)
            .highlight_style(Style::new().yellow())
            .fit(true)
            .on_title_click(move |_, tab| {
                focused.set(tab.to_string());
            })
            .on_direct_focus(move |_, _, _| {
                tab_block.set(Block::bordered().blue().title("Demo"));
            })
            .on_direct_blur(move |_, _, _| {
                tab_block.set(Block::bordered().title("Demo"));
            })
            .on_key_down(
                [
                    key(keys::LEFT, move |_, _| {
                        let tabs = tabs.get();
                        if let Some(prev) = tabs.prev_item(&focused.get()) {
                            focused.set(prev.get_value().to_string());
                        }
                    }),
                    key(keys::RIGHT, move |_, _| {
                        let tabs = tabs.get();
                        if let Some(next) = tabs.next_item(&focused.get()) {
                            focused.set(next.get_value().to_string());
                        }
                    })
                ]
                .bind()
            )
            .render(focused, tabs),
    ]
}

fn inner_tabs() -> impl Render {
    let focused_tab = RwSignal::new("tab1".to_string());
    let tab_block = RwSignal::new(Block::bordered().title("Inner"));

    let tabs = RwSignal::new(KeyedWrappingList::<Tab>(vec![
        Tab::new(line!("Tab1"), "tab1", move || "tab1"),
        Tab::new(line!("Tab2"), "tab2", move || "tab2"),
    ]));

    let on_key_down = move |props: KeyEventProps| {
        let tabs = tabs.get();
        match props.event.code {
            KeyCode::Left => {
                if let Some(prev) = tabs.prev_item(&focused_tab.get()) {
                    focused_tab.set(prev.get_value().to_string());
                }
            }
            KeyCode::Right => {
                if let Some(next) = tabs.next_item(&focused_tab.get()) {
                    focused_tab.set(next.get_value().to_string());
                }
            }
            _ => {}
        }
    };

    row![
        style(borders(Borders::all())),
        TabView::new()
            .header_height(3)
            .block(tab_block)
            .fit(true)
            .highlight_style(Style::new().yellow())
            .on_title_click(move |_, tab| {
                focused_tab.set(tab.to_string());
            })
            .on_direct_focus(move |_, _, _| {
                tab_block.set(Block::bordered().blue().title("Inner"));
            })
            .on_direct_blur(move |_, _, _| {
                tab_block.set(Block::bordered().title("Inner"));
            })
            .on_key_down(on_key_down)
            .render(focused_tab, tabs),
    ]
}
