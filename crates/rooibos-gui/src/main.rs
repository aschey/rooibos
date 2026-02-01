use rooibos_dom::{line, span};
use rooibos_gui::{AppHandle, GuiBackend, run, run_async};
use rooibos_keybind::{key, keys};
use rooibos_reactive::dom::Render;
use rooibos_reactive::graph::signal::signal;
use rooibos_reactive::graph::traits::{Get, Update};
use rooibos_reactive::wgt;
use rooibos_runtime::Runtime;
use rooibos_theme::Stylize;

fn main() {
    run(async_main)
}

#[rooibos_reactive_macros::main]
async fn async_main(app_handle: AppHandle) {
    let runtime = Runtime::initialize(GuiBackend {});
    run_async(app_handle, runtime, (), |_| app()).await;
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key(keys::ENTER, move |_, _| {
            update_count();
        }))
        .on_click(move |_| update_count())
}
