use rooibos::components::Button;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::layout::chars;
use rooibos::reactive::{col, derive_signal, line, span, Render, UpdateLayoutProps};

#[cfg(target_arch = "wasm32")]
#[rooibos::wasm_bindgen(start)]
async fn start() -> Result<(), wasm_bindgen::JsError> {
    use rooibos::runtime::{Runtime, RuntimeSettings};
    use rooibos::xterm_js::WasmBackend;

    let runtime = Runtime::initialize(WasmBackend::default(), app);
    runtime
        .run()
        .await
        .map_err(|e| wasm_bindgen::JsError::new(&e.to_string()))?;
    Ok(())
}

pub fn app() -> impl Render {
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    Button::new()
        .height(chars(3.))
        .width(chars(20.))
        .on_click(move || set_count.update(|c| *c += 1))
        .render(derive_signal!(line!("count ", span!(count.get())).into()))
}
