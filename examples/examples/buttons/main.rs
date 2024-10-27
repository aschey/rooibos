use std::process::ExitCode;

use rooibos::components::{Button, ButtonRef};
use rooibos::dom::{focus_id, text};
use rooibos::keybind::{Bind, KeybindContext, map_handler};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{
    Render, col, derive_signal, height, mount, padding, padding_right, row, wgt, width,
};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::text::Span;
use rooibos::tui::widgets::Paragraph;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

const MIN_SIZE: f32 = 3.;
const MAX_SIZE: f32 = 15.;

fn app() -> impl Render {
    let (block_height, set_block_height) = signal(5.);
    let block_width = derive_signal!(block_height.get() * 2.);

    let adjust_size = move |adjustment: f32| {
        set_block_height.update(|b| {
            *b += adjustment;
            *b = b.clamp(MIN_SIZE, MAX_SIZE);
        });
    };
    let bigger = "bigger";
    let smaller = "smaller";
    let bigger_ref = ButtonRef::new();
    let smaller_ref = ButtonRef::new();

    row![
        props(padding!(1.)),
        col![
            props(width!(20.), padding_right!(2.)),
            button(
                bigger.bold(),
                derive_signal!(block_height.get() < MAX_SIZE),
                bigger_ref,
                move || adjust_size(1.)
            ),
            button(
                smaller.bold(),
                derive_signal!(block_height.get() > MIN_SIZE),
                smaller_ref,
                move || adjust_size(-1.)
            )
        ],
        wgt!(
            props(width!(block_width), height!(block_height)),
            Paragraph::new(format!("{} x {}", block_width.get(), block_height.get()))
                .centered()
                .bg({
                    let height = block_height.get() as f64;
                    Color::from_hsl(18.0 * height, 5.0 * height, 5.0 * height)
                })
        )
    ]
    .on_key_down(
        [
            map_handler("+", move |_, _| {
                focus_id(bigger);
                bigger_ref.click();
            }),
            map_handler("-", move |_, _| {
                focus_id(smaller);
                smaller_ref.click();
            }),
            map_handler("{dec}+", move |_, context: KeybindContext| {
                focus_id(bigger);
                adjust_size(context.keys[0].get_numeric() as f32);
            }),
            map_handler("{dec+}-", move |_, context: KeybindContext| {
                focus_id(smaller);
                adjust_size(-1.0 * context.keys[0].get_numeric() as f32);
            }),
        ]
        .bind(),
    )
}

fn button<F>(
    title: Span<'static>,
    enabled: Signal<bool>,
    button_ref: ButtonRef,
    on_click: F,
) -> impl Render
where
    F: Fn() + Clone + 'static,
{
    row![
        props(height!(3.)),
        Button::new()
            .id(title.to_string())
            .enabled(enabled)
            .element_ref(button_ref)
            .on_click(on_click)
            .render(text!(title))
    ]
}

#[cfg(test)]
mod tests;
