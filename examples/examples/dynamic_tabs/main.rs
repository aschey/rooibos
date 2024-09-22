use rooibos::components::{Button, KeyedWrappingList, Tab, TabView};
use rooibos::dom::{KeyCode, KeyEvent, line, span, text};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::layout::chars;
use rooibos::reactive::{Render, UpdateLayoutProps, mount, padding, row};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::Block;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let focused = RwSignal::new("tab1".to_string());
    let tabs_block = RwSignal::new(Block::bordered().title("Demo"));

    let tabs = RwSignal::new(KeyedWrappingList(vec![
        Tab::new(line!("Tab1"), "tab1", move || " tab1").decorator(line!("✕".red())),
        Tab::new(line!("Tab2"), "tab2", move || " tab2").decorator(line!("✕".red())),
    ]));

    let next_tab = RwSignal::new(3);

    let remove_tab = move |i: usize, tab: &str| {
        tabs.update(|t| {
            t.remove(i);
        });
        if focused.get() == tab {
            let tabs = tabs.get();
            if tabs.is_empty() {
                focused.set("".to_string());
                return;
            }
            let new_idx = (i as isize - 1).max(0);
            focused.set(tabs[new_idx as usize].get_value().to_string());
        }
    };

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
            KeyCode::Char('d') => {
                let focused = focused.get();
                let (i, tab) = tabs
                    .iter()
                    .enumerate()
                    .find(|(_, t)| t.get_value() == focused)
                    .unwrap();
                remove_tab(i, tab.get_value());
            }
            _ => {}
        }
    };

    row![
        props(padding!(1.)),
        TabView::new()
            .header_height(chars(3.))
            .block(tabs_block)
            .highlight_style(Style::new().yellow())
            .fit(true)
            .on_title_click(move |_, tab| {
                focused.set(tab.to_string());
            })
            .on_focus(move |_, _| {
                tabs_block.set(Block::bordered().title("Demo").blue());
            })
            .on_blur(move |_, _| {
                tabs_block.set(Block::bordered().title("Demo"));
            })
            .on_decorator_click(remove_tab)
            .on_key_down(on_key_down)
            .render(focused, tabs),
        Button::new()
            .width(chars(5.))
            .height(chars(3.))
            .on_click(move || {
                tabs.update(|t| {
                    let num = next_tab.get();
                    t.push(
                        Tab::new(line!("Tab", span!(num)), format!("tab{num}"), move || {
                            format!(" tab{num}")
                        })
                        .decorator(line!("✕".red())),
                    );
                    next_tab.update(|t| *t += 1);
                });
                let tabs = tabs.get();
                if tabs.len() == 1 {
                    focused.set(tabs[0].get_value().to_string());
                }
            })
            .render(text!("+".green()))
    ]
}

#[cfg(test)]
mod tests;
