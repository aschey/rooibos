use std::error::Error;

use rooibos::prelude::Constraint::*;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::reactive::wrappers::read::Signal;
use rooibos::runtime::run;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    run().await?;
    Ok(())
}

fn app() -> impl Render {
    Effect::new(move |_| {
        focus_next();
    });

    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    container(
        Length(20),
        Length(5),
        Button::new()
            .on_click(move || set_count.update(|c| *c += 1))
            .render(signal!(Text::from(format!("count {}", count.get())))),
    )
}
