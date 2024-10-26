use std::path::PathBuf;
use std::process::ExitCode;

use rooibos::components::Image;
use rooibos::keybind::{Bind, map_handler};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{GetUntracked, Update};
use rooibos::reactive::{Render, col, height, mount, padding, padding_top, wgt, width};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let image_length = RwSignal::new(10.);
    let image_url = RwSignal::new(PathBuf::from("./examples/assets/cat.jpg"));

    col![
        props(padding!(1.)),
        wgt!(
            props(height!(1.)),
            "Press up to increase image size, down to decrease"
        ),
        col![
            props(
                width!(image_length),
                height!(image_length),
                padding_top!(1.)
            ),
            Image::from_url(image_url).render()
        ]
    ]
    .on_key_down(
        [
            map_handler("<Down>", move |_| {
                if image_length.get_untracked() > 5. {
                    image_length.update(|l| *l -= 1.);
                }
            }),
            map_handler("<Up>", move |_| {
                if image_length.get_untracked() < 20. {
                    image_length.update(|l| *l += 1.);
                }
            }),
            map_handler("t", move |_| {
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
