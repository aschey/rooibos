use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::flex_node::FlexProperty;
use rooibos::reactive::dom::layout::{
    Borders, align_items, borders, center, clear, full, height, justify_content, padding, show,
    width, z_index,
};
use rooibos::reactive::dom::{
    NodeId, Render, UpdateLayoutProps, after_render, focus_id, line, text,
};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::{ReadSignal, signal};
use rooibos::reactive::graph::traits::{Get as _, Set};
use rooibos::reactive::{col, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, max_viewport_height, max_viewport_width};
use rooibos::terminal::DefaultBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let runtime = Runtime::initialize(DefaultBackend::auto().await?);
    runtime.run(|_| app()).await
}

fn app() -> impl Render {
    max_viewport_width(50).unwrap();
    max_viewport_height(20).unwrap();

    let (show_popup, set_show_popup) = signal(false);
    col![
        style(borders(Borders::all()), bounds()),
        row![
            Button::new()
                .on_click(move || set_show_popup.set(true))
                .render(text!("Open Popup")),
        ],
        popup(show_popup, move || set_show_popup.set(false)),
        Button::new().render(text!("bottom text"))
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
        style(
            bounds(),
            z_index(2),
            center_items(),
            justify_content(center()),
            show(show_popup),
        ),
        col![
            style(
                center_items(),
                clear(true),
                padding(1),
                borders(Borders::all())
            ),
            wgt!(line!("popup text")),
            Button::new()
                .on_click(on_close)
                .id(id)
                .render(text!("close"))
        ]
    ]
}

fn bounds() -> impl FlexProperty {
    (width(full()), height(full()))
}

fn center_items() -> impl FlexProperty {
    (align_items(center()), justify_content(center()))
}
