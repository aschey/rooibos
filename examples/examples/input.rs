use rooibos::components::Input;
use rooibos::dom::WidgetState;
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::layout::chars;
use rooibos::reactive::{Render, UpdateLayoutProps, col, mount, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::Block;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let textarea = Input::get_ref();

    let text = textarea.text();
    col![
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
