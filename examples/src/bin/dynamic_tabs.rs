use std::error::Error;
use std::io::Stdout;

use rooibos::components::{Button, Tab, TabList, TabView};
use rooibos::dom::{col, row, Constrainable, EventData, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::{run, start, RuntimeSettings, TerminalSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::text::{Line, Text};
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    start(RuntimeSettings::default(), app);
    run::<Stdout>(TerminalSettings::default()).await?;
    Ok(())
}

fn app() -> impl Render {
    let focused = RwSignal::new("tab1".to_string());

    let tabs = RwSignal::new(TabList(vec![
        Tab::new(Line::from("Tab1"), "tab1", move || "tab1").decorator(Line::from("✕".red())),
        Tab::new(Line::from("Tab2"), "tab2", move || "tab2").decorator(Line::from("✕".red())),
    ]));

    let next_tab = RwSignal::new(3);

    let remove_tab = move |i: usize, tab: &str| {
        tabs.update(|t| {
            t.remove(i);
        });
        if focused.get() == tab {
            let tabs = tabs.get();
            if tabs.is_empty() {
                focused.set("");
                return;
            }
            let new_idx = (i as isize - 1).max(0);
            focused.set(tabs[new_idx as usize].get_value().to_string());
        }
    };

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

    row![
        TabView::new()
            .header_constraint(Length(3))
            .block(Block::bordered().title("Demo"))
            .highlight_style(Style::new().yellow())
            .fit(true)
            .on_title_click(move |_, tab| {
                focused.set(tab);
            })
            .on_decorator_click(remove_tab)
            .on_key_down(on_key_down)
            .render(focused, tabs),
        col![
            row![
                Button::new()
                    .on_click(move || {
                        tabs.update(|t| {
                            let num = next_tab.get();
                            t.push(
                                Tab::new(
                                    Line::from(format!("Tab{num}")),
                                    format!("tab{num}"),
                                    move || format!("tab{num}"),
                                )
                                .decorator(Line::from("✕".red())),
                            );
                            next_tab.update(|t| *t += 1);
                        });
                        let tabs = tabs.get();
                        if tabs.len() == 1 {
                            focused.set(tabs[0].get_value().to_string());
                        }
                    })
                    .render(Text::from("+".green()))
            ]
            .length(3)
        ]
        .length(5)
    ]
}
