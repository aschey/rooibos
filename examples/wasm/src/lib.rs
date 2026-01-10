use rooibos::components::Button;
use rooibos::reactive::col;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};

#[cfg(target_arch = "wasm32")]
#[rooibos::wasm_bindgen(start)]
async fn start() -> Result<(), wasm_bindgen::JsError> {
    use rooibos::runtime::{Runtime, RuntimeSettings};
    use rooibos::web::WasmBackend;
    wasm_tracing::set_as_global_default();

    let runtime = Runtime::initialize(WasmBackend::default());
    runtime
        .run(|_| app())
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
        .render(move || line!("count ", count.get().to_string()).into())
}
