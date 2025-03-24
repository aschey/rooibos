use std::process::ExitCode;

use rooibos::components::{Button, ButtonRef};
use rooibos::keybind::{Bind, KeybindContext, key, keys};
use rooibos::reactive::dom::div::taffy::Overflow;
use rooibos::reactive::dom::layout::{full, height, overflow, padding_right, width};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, span, text, try_focus_id};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{col, derive_signal, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::palette::Hsl;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::text::Span;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

const MIN_SIZE: u32 = 3;
const MAX_SIZE: u32 = 15;

fn app() -> impl Render {
    let (block_height, set_block_height) = signal(5u32);
    let block_width = derive_signal!(block_height.get() * 2);

    let adjust_size = move |adjustment: i32| {
        set_block_height.update(|b| {
            *b = (*b as i32 + adjustment) as u32;
            *b = (*b).clamp(MIN_SIZE, MAX_SIZE);
        });
    };
    let bigger = "bigger";
    let smaller = "smaller";
    let bigger_ref = ButtonRef::new();
    let smaller_ref = ButtonRef::new();

    row![
        style(height(full()), overflow(Overflow::Scroll)),
        col![
            style(width(15), padding_right(2)),
            button(
                bigger.bold(),
                derive_signal!(block_height.get() < MAX_SIZE),
                bigger_ref,
                move || adjust_size(1)
            ),
            button(
                smaller.bold(),
                derive_signal!(block_height.get() > MIN_SIZE),
                smaller_ref,
                move || adjust_size(-1)
            )
        ],
        wgt!(
            style(width(block_width), height(block_height)),
            text!(span!("{} x {}", block_width.get(), block_height.get()))
                .centered()
                .bg({
                    let height = block_height.get() as f32;
                    Color::from_hsl(Hsl::new(
                        18.0 * height,
                        5.0 * height / 100.,
                        5.0 * height / 100.,
                    ))
                })
        )
    ]
    .on_key_down(
        [
            key("+", move |_, _| {
                if try_focus_id(bigger).is_ok() {
                    bigger_ref.click();
                }
            }),
            key("-", move |_, _| {
                if try_focus_id(smaller).is_ok() {
                    smaller_ref.click();
                }
            }),
            //"{dec+}+"
            key(
                keys::combine([keys::Key::decimal('+'), '+'.into()]),
                move |_, context: KeybindContext| {
                    if try_focus_id(bigger).is_ok() {
                        adjust_size(context.keys[0].get_numeric() as i32);
                    }
                },
            ),
            //"{dec+}-"
            key(
                keys::combine([keys::Key::decimal('+'), '-'.into()]),
                move |_, context: KeybindContext| {
                    if try_focus_id(smaller).is_ok() {
                        adjust_size(-(context.keys[0].get_numeric() as i32));
                    }
                },
            ),
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
    Button::new()
        .id(title.to_string())
        .centered()
        .enabled(enabled)
        .element_ref(button_ref)
        .on_click(on_click)
        .render(text!(title))
}

mod tests;
