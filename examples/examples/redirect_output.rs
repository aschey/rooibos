use std::error::Error;
use std::io::{IsTerminal, Stderr, stdout};
use std::process::ExitCode;

use rooibos::components::{ListView, WrappingList};
use rooibos::dom::{KeyCode, KeyEventProps, text};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set, With};
use rooibos::reactive::{Render, col, height, mount, wgt};
use rooibos::runtime::{Runtime, RuntimeSettings, exit};
use rooibos::terminal::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::tui::Viewport;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::ListItem;

type Result = std::result::Result<ExitCode, Box<dyn Error>>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    if stdout().is_terminal() {
        return Err("Try redirecting the output. Ex: out=$(cargo run --example=redirect_output)")?;
    }

    mount(app);
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default()
            .viewport(Viewport::Inline(6))
            .show_final_output(false),
        CrosstermBackend::new(TerminalSettings::<Stderr>::new().alternate_screen(false)),
    );
    Ok(runtime.run().await?)
}

fn app() -> impl Render {
    let selected = RwSignal::new(Some(0));
    let item_text = ["Item 1", "Item 2", "Item 3"];
    let items = RwSignal::new(WrappingList(
        item_text.iter().map(|t| ListItem::new(*t)).collect(),
    ));

    let on_key_down = move |props: KeyEventProps| {
        let selected_idx = selected.get().unwrap();
        match props.event.code {
            KeyCode::Down => {
                items.with(|i| {
                    selected.set(i.next_index(selected_idx));
                });
            }
            KeyCode::Up => {
                items.with(|i| {
                    selected.set(i.prev_index(selected_idx));
                });
            }
            KeyCode::Enter => {
                println!("{}", item_text[selected_idx]);
                exit();
            }
            _ => {}
        }
    };
    col![
        wgt!(props(height!(2.)), text!("Select an item".bold())),
        ListView::new()
            .on_item_click(move |i, _| {
                selected.set(Some(i));
            })
            .on_key_down(on_key_down)
            .highlight_style(Style::new().green())
            .render(selected, items)
    ]
}
