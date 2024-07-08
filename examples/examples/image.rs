use std::error::Error;
use std::io::Stdout;
use std::path::PathBuf;

use rooibos::components::Image;
use rooibos::dom::{col, row, Constrainable, KeyCode, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let length = RwSignal::new(5);

    let keypress = use_keypress();

    let image_url = RwSignal::new(PathBuf::from("./examples/assets/cat.jpg"));

    Effect::new(move |_| {
        if let Some(key) = keypress.get() {
            if key.code == KeyCode::Down && length.get_untracked() > 1 {
                length.update(|l| *l -= 1);
            } else if key.code == KeyCode::Up && length.get_untracked() < 10 {
                length.update(|l| *l += 1);
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

    col![row![Image::from_url(image_url).render()].length(length)]
}
