use std::error::Error;

use rooibos::prelude::Constraint::*;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::run;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(counters);
    run().await?;
    Ok(())
}

fn counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Up {
            set_count.update(|c| *c += 1);
        }
        if key_event.code == KeyCode::Down {
            set_count.update(|c| *c -= 1);
        }
    };

    widget_ref!(Block::new().title(format!("count: {}", count.get())))
        .id(id.to_string())
        .focusable(true)
        .constraint(constraint)
        .on_key_down(key_down)
}

fn counters() -> impl Render {
    Effect::new(move |_| {
        focus_next();
    });

    col![{ (0..5).map(|i| counter(i, Length(2))).collect::<Vec<_>>() }]
}
