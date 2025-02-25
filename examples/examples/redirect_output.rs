use std::error::Error;
use std::io::{IsTerminal, stdout};
use std::process::ExitCode;

use rooibos::components::{ListView, WrappingList};
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::height;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, text};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set, With};
use rooibos::reactive::{col, wgt};
use rooibos::runtime::{Runtime, RuntimeSettings, exit};
use rooibos::terminal::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::tui::Viewport;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::ListItem;

type Result = std::result::Result<ExitCode, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result {
    if stdout().is_terminal() {
        return Err("Try redirecting the output. Ex: out=$(cargo run --example=redirect_output)")?;
    }

    let res = Runtime::initialize_with(
        RuntimeSettings::default()
            .viewport(Viewport::Inline(6))
            .show_final_output(false),
        CrosstermBackend::new(TerminalSettings::auto().alternate_screen(false)),
    )
    .run(app)
    .await?;
    Ok(res)
}

fn app() -> impl Render {
    let selected = RwSignal::new(Some(0));
    let item_text = ["Item 1", "Item 2", "Item 3"];
    let items = RwSignal::new(WrappingList(
        item_text.iter().map(|t| ListItem::new(*t)).collect(),
    ));

    col![
        wgt!(style(height(2)), text!("Select an item".bold())),
        ListView::new()
            .height(3)
            .on_item_click(move |i, _| {
                selected.set(Some(i));
            })
            .on_key_down(
                [
                    key(keys::DOWN, move |_, _| {
                        let selected_idx = selected.get().unwrap();
                        items.with(|i| {
                            selected.set(i.next_index(selected_idx));
                        });
                    }),
                    key(keys::UP, move |_, _| {
                        let selected_idx = selected.get().unwrap();
                        items.with(|i| {
                            selected.set(i.prev_index(selected_idx));
                        });
                    }),
                    key(keys::ENTER, move |_, _| {
                        let selected_idx = selected.get().unwrap();
                        println!("{}", item_text[selected_idx]);
                        exit();
                    })
                ]
                .bind()
            )
            .highlight_style(Style::new().green())
            .render(selected, items)
    ]
}
