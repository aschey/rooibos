use std::error::Error;

use crossterm::event::KeyCode;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::runtime::{run, use_keypress};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(|| view!(<App/>));
    run().await?;
    Ok(())
}

#[component]
fn App() -> impl Render {
    let child2_id = RwSignal::new(0);

    view! {
        <Col>
            <Router>
                <Route path="/">
                    {move || view!(<Child0/>)}
                </Route>
                <Route path="/child1">
                    {move || view!(<Child1 child2_id=child2_id/>)}
                </Route>
                <Route path="/child2/{id}">
                    {move || view!(<Child2/>)}
                </Route>
        </Router>
    </Col>
    }
}

#[component]
fn Child0() -> impl Render {
    let router = use_router();

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                router.push("/child1?id=1");
            }
        }
    });

    view! {
        <Paragraph>
            "child0"
        </Paragraph>
    }
}

#[component]
fn Child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.use_query("id");

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                router.push(format!("/child2/{}", child2_id.get_untracked()));
                child2_id.update(|id| *id += 1);
            }
        }
    });

    view! {
        <Paragraph>
            {format!("child1 id={}", id.get().unwrap())}
        </Paragraph>
    }
}

#[component]
fn Child2() -> impl Render {
    let router = use_router();
    let id = router.use_param("id");

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                router.pop();
            }
        }
    });

    view! {
        <Paragraph>
            {format!("child2 id={}", id.get().unwrap())}
        </Paragraph>
    }
}
