use std::path::PathBuf;
use std::process::ExitCode;

use rooibos::components::Image;
use rooibos::dom::KeyCode;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, Update};
use rooibos::reactive::{Render, col, height, mount, padding, padding_top, wgt, width};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, use_keypress};
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

    let keypress = use_keypress();

    let image_url = RwSignal::new(PathBuf::from("./examples/assets/cat.jpg"));

    Effect::new(move || {
        if let Some(key) = keypress.get() {
            if key.code == KeyCode::Down && image_length.get_untracked() > 5. {
                image_length.update(|l| *l -= 1.);
            } else if key.code == KeyCode::Up && image_length.get_untracked() < 20. {
                image_length.update(|l| *l += 1.);
            } else if key.code == KeyCode::Char('t') {
                image_url.update(|i| {
                    if i.to_string_lossy() == "./examples/assets/cat.jpg" {
                        *i = PathBuf::from("./examples/assets/cat2.jpg")
                    } else {
                        *i = PathBuf::from("./examples/assets/cat.jpg")
                    }
                });
            }
        }
    });
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
}
