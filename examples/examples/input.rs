use std::process::ExitCode;

use rooibos::components::Input;
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, WidgetState, line, mount};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, padding, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::Block;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let textarea = Input::get_ref();

    let text = textarea.text();
    col![
        props(padding!(1.)),
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
                textarea.delete_line();
            })
            .render(textarea),
        wgt!(line!("You typed: ", text.get().bold()))
    ]
}
