use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::run;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(counter);
    run().await?;
    Ok(())
}

fn counter() -> impl Render {
    let (count, set_count) = signal(0);

    Effect::new(move |_| {
        focus_next();
    });

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    paragraph(move || ParagraphProps::new(format!("count {}", count.get())))
        .focusable(true)
        .on_key_down(key_down)
}
