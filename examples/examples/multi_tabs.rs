use std::error::Error;

use rooibos::components::{KeyedWrappingList, Tab, TabView};
use rooibos::dom::{block, col, line, row, Constrainable, EventData, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(RuntimeSettings::default(), CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let focused = RwSignal::new("tab1".to_string());
    let tab_block = RwSignal::new(Block::bordered().title("Demo"));

    let tabs = RwSignal::new(KeyedWrappingList::<Tab>(vec![
        Tab::new(line!("Tab1"), "tab1", move || "tab1"),
        Tab::new(line!("Tab2"), "tab2", inner_tabs),
        Tab::new(line!("Tab3"), "tab3", move || "tab3"),
    ]));

    let on_key_down = move |key_event: KeyEvent, _: EventData| {
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

    row![
        col![
            TabView::new()
                .header_constraint(Length(3))
                .block(tab_block)
                .highlight_style(Style::new().yellow())
                .fit(true)
                .on_title_click(move |_, tab| {
                    focused.set(tab.to_string());
                })
                .on_focus(move |_, _| {
                    tab_block.set(Block::bordered().blue().title("Demo"));
                })
                .on_blur(move |_, _| {
                    tab_block.set(Block::bordered().title("Demo"));
                })
                .on_key_down(on_key_down)
                .render(focused, tabs),
        ]
        .block(Block::bordered())
        .percentage(50)
    ]
}

fn inner_tabs() -> impl Render {
    let focused_tab = RwSignal::new("tab1".to_string());
    let tab_block = RwSignal::new(Block::bordered().title("Inner"));

    let tabs = RwSignal::new(KeyedWrappingList::<Tab>(vec![
        Tab::new(line!("Tab1"), "tab1", move || "tab1"),
        Tab::new(line!("Tab2"), "tab2", move || "tab2"),
    ]));

    let on_key_down = move |key_event: KeyEvent, _: EventData| {
        let tabs = tabs.get();
        match key_event.code {
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
        props(block(Block::bordered())),
        TabView::new()
            .header_constraint(Length(3))
            .block(tab_block)
            .highlight_style(Style::new().yellow())
            .on_title_click(move |_, tab| {
                focused_tab.set(tab.to_string());
            })
            .on_focus(move |_, _| {
                tab_block.set(Block::bordered().blue().title("Inner"));
            })
            .on_blur(move |_, _| {
                tab_block.set(Block::bordered().title("Inner"));
            })
            .on_key_down(on_key_down)
            .render(focused_tab, tabs),
    ]
}
