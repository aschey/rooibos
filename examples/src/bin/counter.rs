use std::error::Error;

use crossterm::event::KeyCode;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, use_keypress};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(|| view!(<Counter/>));
    run().await?;
    Ok(())
}

#[component]
fn Counter() -> impl Render {
    let (count, set_count) = signal(0);
    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                set_count.update(|c| *c += 1);
            }
        }
    });

    view! {
        <Col>
            {move || format!("count {}", count.get())}
        </Col>
    }
}
