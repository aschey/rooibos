use rooibos::components::{Button, for_each};
use rooibos::dom::{KeyCode, line, span, text};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::layout::{chars, height};
use rooibos::reactive::{
    Render, UpdateLayoutProps, col, height, margin, max_width, mount, padding, padding_left, row,
    wgt, width,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, use_keypress};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::Paragraph;

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn counter(
    row_height: Signal<taffy::Dimension>,
    on_remove: impl Fn() + Clone + 'static,
) -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    row![
        props(height(row_height), padding_left!(1.)),
        Button::new()
            .width(chars(6.))
            .on_click(decrease)
            .render(text!("-1")),
        Button::new()
            .width(chars(6.))
            .on_click(increase)
            .render(text!("+1")),
        Button::new()
            .width(chars(5.))
            .on_click(on_remove)
            .render(text!("x".red())),
        wgt!(
            props(width!(10.), margin!(1.)),
            Paragraph::new(line!("count: ".bold(), span!(count.get()).cyan()))
        )
        .on_click(move |_, _, _| increase())
        .on_right_click(move |_, _, _| decrease())
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

    col![
        props(max_width!(50.), padding!(1.)),
        row![
            props(height!(3.)),
            Button::new()
                .on_click(add_counter)
                .render(text!("Add Counter")),
        ],
        for_each(
            move || ids.get(),
            |k| *k,
            move |i| counter(chars(3.), move || remove_id(i))
        )
    ]
    .id("root")
}

#[cfg(test)]
mod tests;
