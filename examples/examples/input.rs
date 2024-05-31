use std::error::Error;
use std::io::Stdout;

use rooibos::components::Input;
use rooibos::dom::{col, widget_ref, Constrainable, Render, WidgetState};
use rooibos::reactive::traits::Get;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let textarea = Input::get_ref();

    let text = textarea.text();
    col![
        Input::default()
            .block(|state| Block::bordered()
                .fg(if state == WidgetState::Focused {
                    Color::Blue
                } else {
                    Color::default()
                })
                .title("Input")
                .into())
            .placeholder_text("Enter some text")
            .length(3)
            .on_submit(move |_| {
                textarea.delete_line_by_head();
            })
            .render(textarea),
        widget_ref!(format!("You typed {}", text.get()))
    ]
}
