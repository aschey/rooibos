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
    mount(|| view!(<App/>));
    run().await?;
    Ok(())
}

#[component]
fn App() -> impl Render {
    Effect::new(move |_| {
        focus_next();
    });

    view! {
        <col>
            <CounterButton/>
            <CounterButton/>
        </col>
    }
}

#[component]
fn CounterButton() -> impl Render {
    let (count, set_count) = signal(0);

    view! {
        <Container h_constraint=Length(20) v_constraint=Length(5)>
            {view! {
                <Button on_click=move || set_count.update(|c| *c +=1)>
                    {move || format!("count {}", count.get())}
                </Button>
            }}
        </Container>
    }
}
