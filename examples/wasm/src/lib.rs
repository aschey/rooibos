use rooibos::components::Button;
use rooibos::dom::layout::chars;
use rooibos::dom::{derive_signal, flex_col, line, span, Render, UpdateLayoutProps};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};

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
    flex_col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    Button::new()
        .height(chars(3.))
        .width(chars(20.))
        .on_click(move || set_count.update(|c| *c += 1))
        .render(derive_signal!(line!("count ", span!(count.get())).into()))
}
