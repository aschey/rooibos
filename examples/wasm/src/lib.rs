use rooibos::components::Button;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, derive_signal};

#[cfg(target_arch = "wasm32")]
#[rooibos::wasm_bindgen(start)]
async fn start() -> Result<(), wasm_bindgen::JsError> {
    use rooibos::runtime::{Runtime, RuntimeSettings};
    use rooibos::xterm_js::WasmBackend;

    let runtime = Runtime::initialize(WasmBackend::default());
    runtime
        .run(app)
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
        .centered()
        .padding_x(1)
        .on_click(move || set_count.update(|c| *c += 1))
        .render(derive_signal!(
            line!("count ", count.get().to_string()).into()
        ))
}
