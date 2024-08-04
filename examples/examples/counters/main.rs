use std::error::Error;

use rooibos::components::{for_each, Button};
use rooibos::dom::{
    col, constraint, length, row, text, widget_ref, Constrainable, KeyCode, Render,
};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime, RuntimeSettings};
use rooibos::tui::layout::Constraint::{self, *};
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::{Block, Padding, Paragraph};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(RuntimeSettings::default(), CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn counter(
    id: i32,
    on_remove: impl Fn() + Clone + 'static,
    counter_constraint: Constraint,
) -> impl Render {
    let (count, set_count) = signal(0);
    let default_padding = Padding {
        left: 1,
        top: 1,
        ..Default::default()
    };
    let block = RwSignal::new(Block::default().padding(default_padding));

    let update_count = move |change: i32| set_count.update(|c| *c += change);

    row![
        props(constraint(counter_constraint)),
        Button::new()
            .length(6)
            .on_click(move || update_count(-1))
            .render(text!("-1")),
        Button::new()
            .length(6)
            .on_click(move || update_count(1))
            .render(text!("+1")),
        Button::new()
            .length(5)
            .on_click(on_remove)
            .render(text!("x".red())),
        widget_ref![
            props(length(15)),
            Paragraph::new(format!("count: {}", count.get())).block(block.get())
        ]
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
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Char('a') {
                add_counter();
            }
        }
    });

    col![
        row![
            props(length(3)),
            Button::new()
                .on_click(add_counter)
                .length(20)
                .render(text!("Add Counter"))
        ],
        for_each(
            move || ids.get(),
            |k| *k,
            move |i| counter(i, move || remove_id(i), Length(3))
        )
    ]
    .id("root")
}

#[cfg(test)]
mod tests;
