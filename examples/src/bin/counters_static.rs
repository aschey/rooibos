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
    mount(|| view!(<Counters/>));
    run().await?;
    Ok(())
}

#[component]
fn Counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Up {
            set_count.update(|c| *c += 1);
        }
        if key_event.code == KeyCode::Down {
            set_count.update(|c| *c -= 1);
        }
    };

    view! {
        <block
            v:id=id.to_string()
            v:focusable
            title=format!("count: {}", count.get())
            v:constraint=constraint
            on:key_down=key_down
        />
    }
}

#[component]
fn Counters() -> impl Render {
    Effect::new(move |_| {
        focus_next();
    });

    view! {
        <col>
            {(0..5).map(|i| {
                view! {
                    <Counter id=i constraint=Length(2)/>
                }
            }).collect::<Vec<_>>()}
        </col>
    }
}
