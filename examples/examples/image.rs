use std::error::Error;
use std::path::PathBuf;

use rooibos::components::Image;
use rooibos::dom::{col, length, row, KeyCode, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(RuntimeSettings::default(), CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let image_length = RwSignal::new(5);

    let keypress = use_keypress();

    let image_url = RwSignal::new(PathBuf::from("./examples/assets/cat.jpg"));

    Effect::new(move |_| {
        if let Some(key) = keypress.get() {
            if key.code == KeyCode::Down && image_length.get_untracked() > 1 {
                image_length.update(|l| *l -= 1);
            } else if key.code == KeyCode::Up && image_length.get_untracked() < 10 {
                image_length.update(|l| *l += 1);
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

    col![row![
        props(length(image_length)),
        Image::from_url(image_url).render()
    ]]
}
