use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::flex_node::FlexProperty;
use rooibos::reactive::dom::layout::{align_items, block, chars, justify_content, show};
use rooibos::reactive::dom::{
    NodeId, Render, UpdateLayoutProps, after_render, focus_id, line, mount, text,
};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get as _, Set};
use rooibos::reactive::{col, height, margin_left, max_height, max_width, wgt, width};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Block;
use taffy::{AlignItems, JustifyContent};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
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
    let id = NodeId::new_auto();

    Effect::new(move || {
        if show_popup.get() {
            after_render(move || {
                focus_id(id);
            });
        };
    });

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
                .id(id)
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
