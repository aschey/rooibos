use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, use_keypress};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(counters);
    run().await?;
    Ok(())
}

fn counter(id: u32, constraint: Constraint) -> impl Render {
    let count = RwSignal::new(id);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                count.update(|c| *c += 1);
            }
        }
    });

    block(move || BlockProps::default().title(format!("count: {}", count.get())))
        .constraint(constraint)
}

fn counters() -> impl Render {
    let count = RwSignal::new(1);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                count.update(|c| *c += 1);
            }
            if term_signal.code == KeyCode::Down {
                count.update(|c| *c -= 1);
            }
        }
    });

    col().child(for_each(move || {
        ForEachProps::builder()
            .each(move || (1..count.get() + 1))
            .key(|k| *k)
            .children(|i| counter(i, Constraint::Length(2)))
            .build()
    }))
}
