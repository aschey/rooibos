use std::process::ExitCode;

use rooibos::reactive::dom::layout::{
    Borders, borders, clear, height, padding_left, padding_top, width, z_index,
};
use rooibos::reactive::dom::{Render, text};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::Update;
use rooibos::reactive::{col, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(app)
        .await
}

fn app() -> impl Render {
    let (z_index1, set_z_index1) = signal(1);
    let (z_index2, set_z_index2) = signal(2);
    col![
        wgt!(
            style(
                clear(true),
                z_index(z_index1),
                width(20),
                height(3),
                borders(Borders::all())
            ),
            text!("block one")
        )
        .on_click(move |_| {
            set_z_index1.update(|z| *z += 1);
        }),
        col![
            style(z_index(z_index2), padding_left(3), padding_top(1)),
            wgt!(
                style(clear(true), width(23), height(6), borders(Borders::all())),
                text!("block two")
            )
            .on_click(move |_| {
                set_z_index2.update(|z| *z += 1);
            })
        ]
    ]
}
