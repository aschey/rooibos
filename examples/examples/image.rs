use std::path::PathBuf;

use rooibos::components::Image;
use rooibos::dom::KeyCode;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, Update};
use rooibos::reactive::{col, height, mount, width, Render};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{use_keypress, Runtime};
use rooibos::terminal::crossterm::CrosstermBackend;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
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
        props(width!(image_length), height!(image_length)),
        Image::from_url(image_url).render()
    ]
}
