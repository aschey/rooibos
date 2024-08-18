use std::error::Error;

use rooibos::components::Input;
use rooibos::dom::layout::chars;
use rooibos::dom::{flex_col, wgt, Render, UpdateLayoutProps, WidgetState};
use rooibos::reactive::traits::Get;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let textarea = Input::get_ref();

    let text = textarea.text();
    flex_col![
        Input::default()
            .block(|state| Block::bordered()
                .fg(Color::Blue)
                .border_set(if state == WidgetState::Focused {
                    border::PLAIN
                } else {
                    border::EMPTY
                })
                .title("Input")
                .into())
            .placeholder_text("Enter some text")
            .height(chars(3.))
            .on_submit(move |_| {
                textarea.delete_line_by_head();
            })
            .render(textarea),
        wgt!(format!("You typed {}", text.get()))
    ]
}
