use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::flex_node::FlexProperty;
use rooibos::reactive::dom::layout::{
    Borders, align_items, borders, justify_content, show, z_index,
};
use rooibos::reactive::dom::{NodeId, Render, after_render, focus_id, line, text};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get as _, Set};
use rooibos::reactive::{col, height, max_height, max_width, padding, row, wgt, width};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use taffy::{AlignItems, JustifyContent};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run(app).await
}

fn app() -> impl Render {
    let (show_popup, set_show_popup) = signal(false);
    col![
        props(borders(Borders::all()), bounds()),
        row![
            Button::new()
                .on_click(move || set_show_popup.set(true))
                .render(text!("Open Popup")),
        ],
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
            bounds(),
            z_index(2),
            center_items(),
            justify_content(JustifyContent::Center),
            show(show_popup),
        ),
        col![
            props(center_items(), padding!(1.), borders(Borders::all())),
            wgt!(line!("popup text")),
            Button::new()
                .on_click(on_close)
                .id(id)
                .render(text!("close"))
        ]
    ]
}

fn bounds() -> impl FlexProperty {
    (
        width!(100.%),
        height!(100.%),
        max_width!(50.),
        max_height!(20.),
    )
}

fn center_items() -> impl FlexProperty {
    (
        align_items(AlignItems::Center),
        justify_content(JustifyContent::Center),
    )
}
