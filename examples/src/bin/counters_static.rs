use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, use_keypress};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(|| view!(<Counters/>));
    run().await?;
    Ok(())
}

#[component]
fn Counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                set_count.update(|c| *c += 1);
            }
            if term_signal.code == KeyCode::Down {
                set_count.update(|c| *c -= 1);
            }
        }
    });

    view! {
        <Block v:id=id.to_string() title=format!("count: {}", count.get()) v:constraint=constraint/>
    }
}

#[component]
fn Counters() -> impl Render {
    view! {
        <Col>
        {(0..5).map(|i| {
            view! {
                <Counter id=i constraint=Constraint::Length(2)/>
            }
        }).collect::<Vec<_>>()}
        </Col>
    }
}
