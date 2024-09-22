use rooibos::components::Button;
use rooibos::dom::{line, text};
use rooibos::reactive::flex_node::FlexProperty;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::Set;
use rooibos::reactive::layout::{align_items, block, chars, justify_content, show};
use rooibos::reactive::{
    Render, UpdateLayoutProps, col, height, margin_left, max_height, max_width, mount, wgt, width,
};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Block;
use taffy::{AlignItems, JustifyContent};

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (show_popup, set_show_popup) = signal(false);
    col![
        props(max_width!(50.), max_height!(20.)),
        Button::new()
            .width(chars(14.))
            .height(chars(3.))
            .on_click(move || set_show_popup.set(true))
            .render(text!("open popup")),
        popup(show_popup, move || set_show_popup.set(false))
    ]
}

fn popup(show_popup: ReadSignal<bool>, on_close: impl Fn() + Clone + 'static) -> impl Render {
    col![
        props(
            width!(100.%),
            height!(100.%),
            center_items(),
            justify_content(JustifyContent::Center),
            show(show_popup)
        ),
        col![
            props(
                max_width!(21.),
                max_height!(8.),
                center_items(),
                block(Block::bordered())
            ),
            wgt!(props(margin_left!(1.)), line!("popup text")),
            Button::new()
                .height(chars(3.))
                .width(chars(9.))
                .on_click(on_close)
                .render(text!("close"))
        ]
    ]
}

fn center_items() -> impl FlexProperty {
    (
        align_items(AlignItems::Center),
        justify_content(JustifyContent::Center),
    )
}
