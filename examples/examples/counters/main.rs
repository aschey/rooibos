use std::error::Error;

use rooibos::components::{for_each, Button};
use rooibos::dom::layout::{align_items, chars, height, padding, Height};
use rooibos::dom::{
    col, constraint, div, flex_col, flex_row, height, length, margin, padding, padding_x, props,
    row, text, wgt, width, Constrainable, KeyCode, LayoutProps, Render, UpdateLayoutProps,
};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::reactive::wrappers::read::Signal;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime};
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::{Block, Padding, Paragraph};
use taffy::AlignItems;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn counter(
    row_height: Signal<taffy::Dimension>,
    id: i32,
    on_remove: impl Fn() + Clone + 'static,
) -> impl Render {
    let (count, set_count) = signal(0);
    let default_padding = Padding {
        left: 1,
        top: 1,
        ..Default::default()
    };
    let block = RwSignal::new(Block::default().padding(default_padding));

    let update_count = move |change: i32| set_count.update(|c| *c += change);

    flex_row![
        props(height(row_height)),
        Button::new()
            .width(chars(6.))
            .on_click(move || update_count(-1))
            .render(text!("-1")),
        Button::new()
            .width(chars(6.))
            .on_click(move || update_count(1))
            .render(text!("+1")),
        Button::new()
            .width(chars(5.))
            .on_click(on_remove)
            .render(text!("x".red())),
        wgt!(
            props(width!(15.)),
            Paragraph::new(format!("count: {}", count.get())).block(block.get())
        )
        .on_click(move |_, _| update_count(1))
        .id(id.to_string())
    ]
}

fn app() -> impl Render {
    let (ids, set_ids) = signal(vec![]);
    let (next_id, set_next_id) = signal(0);

    let remove_id = move |id: i32| {
        set_ids.update(|ids| ids.retain(|i| *i != id));
    };

    let add_counter = move || {
        set_ids.update(|s| s.push(next_id.get_untracked()));
        set_next_id.update(|n| *n += 1);
    };

    let term_signal = use_keypress();
    Effect::new(move || {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Char('a') {
                add_counter();
            }
        }
    });

    flex_col![
        props(align_items(AlignItems::Center), padding_x!(25.%)),
        flex_row![
            props(height!(3.)),
            Button::new()
                .on_click(add_counter)
                .render(text!("Add Counter")),
        ],
        div![for_each(
            move || ids.get(),
            |k| *k,
            move |i| counter(chars(3.), i, move || remove_id(i))
        )]
    ]
    .id("root")
}

#[cfg(test)]
mod tests;
