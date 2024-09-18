use rooibos::components::Button;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{
    col, derive_signal, height, mount, padding_right, row, text, wgt, width, Render,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::text::Span;
use rooibos::tui::widgets::Paragraph;

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

const MIN_SIZE: f32 = 3.;
const MAX_SIZE: f32 = 15.;

fn app() -> impl Render {
    let (block_height, set_block_height) = signal(5.);
    let block_width = derive_signal!(block_height.get() * 2.);

    let adjust_size = move |adjustment: f32| {
        set_block_height.update(|b| {
            *b += adjustment;
        });
    };

    row![
        col![
            props(width!(20.), padding_right!(2.)),
            button(
                "bigger".bold(),
                derive_signal!(block_height.get() >= MAX_SIZE),
                move || adjust_size(1.)
            ),
            button(
                "smaller".bold(),
                derive_signal!(block_height.get() <= MIN_SIZE),
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
}

fn button<F>(title: Span<'static>, disabled: Signal<bool>, on_click: F) -> impl Render
where
    F: Fn() + Clone + 'static,
{
    row![
        props(height!(3.)),
        Button::new()
            .disabled(disabled)
            .on_click(on_click)
            .render(text!(title))
    ]
}
#[cfg(test)]
mod tests;
