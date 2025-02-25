use std::path::PathBuf;
use std::process::ExitCode;

use rooibos::components::Image;
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::Render;
use rooibos::reactive::dom::layout::{height, padding, padding_top, width};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{GetUntracked, Update};
use rooibos::reactive::{col, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let image_length = RwSignal::new(10);
    let image_url = RwSignal::new(PathBuf::from("./examples/assets/cat.jpg"));

    col![
        props(padding(1)),
        wgt!(
            props(height(1)),
            "Press up to increase image size, down to decrease"
        ),
        col![
            props(width(image_length), height(image_length), padding_top(1)),
            Image::from_url(image_url).render()
        ]
    ]
    .on_key_down(
        [
            key(keys::DOWN, move |_, _| {
                if image_length.get_untracked() > 5 {
                    image_length.update(|l| *l = (*l as i32 - 1) as u32);
                }
            }),
            key(keys::UP, move |_, _| {
                if image_length.get_untracked() < 20 {
                    image_length.update(|l| *l += 1);
                }
            }),
            key("t", move |_, _| {
                image_url.update(|i| {
                    if i.to_string_lossy() == "./examples/assets/cat.jpg" {
                        *i = PathBuf::from("./examples/assets/cat2.jpg")
                    } else {
                        *i = PathBuf::from("./examples/assets/cat.jpg")
                    }
                });
            }),
        ]
        .bind(),
    )
}
