use std::process::ExitCode;

use ratatui::style::Color;
use rooibos::reactive::dom::layout::effect;
use rooibos::reactive::dom::{Render, text};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::Set;
use rooibos::reactive::tachyonfx::fx::{self, Glitch};
use rooibos::reactive::tachyonfx::{EffectTimer, Interpolation, IntoEffect, Motion, SimpleRng};
use rooibos::reactive::{row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, wasm_compat};
use rooibos::terminal::DefaultBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let (current_effect, set_current_effect) = signal(fx::sequence(&[
        fx::ping_pong(fx::sweep_in(
            Motion::LeftToRight,
            10,
            0,
            Color::DarkGray,
            EffectTimer::from_ms(2000, Interpolation::QuadIn),
        )),
        fx::coalesce((800, Interpolation::SineOut)),
    ]));
    wasm_compat::spawn_local(async move {
        wasm_compat::sleep(std::time::Duration::from_secs(7)).await;
        let glitch = Glitch::builder()
            .rng(SimpleRng::default())
            .action_ms(200..400)
            .action_start_delay_ms(0..1)
            .cell_glitch_ratio(1.0)
            .build()
            .into_effect();
        set_current_effect.set(glitch);
    });

    row![
        style(effect(current_effect)),
        wgt!(text!("my super cool effect"))
    ]
}
