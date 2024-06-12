use rooibos::components::Button;
use rooibos::dom::{col, derive_signal, row, Constrainable, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::tui::text::Text;

#[cfg(target_arch = "wasm32")]
#[rooibos::main(wasm)]
async fn start() -> Result<(), wasm_bindgen::JsError> {
    use rooibos::runtime::{Runtime, RuntimeSettings};
    use rooibos::xterm_js::WasmBackend;

    let runtime = Runtime::initialize(RuntimeSettings::default(), WasmBackend::default(), app);
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
    row![
        Button::new()
            .length(20)
            .on_click(move || set_count.update(|c| *c += 1))
            .render(derive_signal!(Text::from(format!("count {}", count.get()))))
    ]
    .length(3)
}
