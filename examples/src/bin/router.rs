use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, GetUntracked, Update};
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
    let child2_id = RwSignal::new(0);

    Effect::new(move |_| {
        focus_next();
    });

    view! {
        <col>
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
    </col>
    }
}

#[component]
fn Child0() -> impl Render {
    let router = use_router();

    Effect::new(move |_| {
        focus_id("child0");
    });

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Enter {
            router.push("/child1?id=1");
        }
    };

    view! {
        <paragraph v:id="child0" v:focusable on:key_down=key_down>
            "child0"
        </paragraph>
    }
}

#[component]
fn Child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.use_query("id");

    Effect::new(move |_| {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Enter {
            router.push(format!("/child2/{}", child2_id.get_untracked()));
            child2_id.update(|id| *id += 1);
        }
    };

    view! {
        <paragraph v:id="child1" v:focusable on:key_down=key_down>
            {format!("child1 id={}", id.get().unwrap())}
        </paragraph>
    }
}

#[component]
fn Child2() -> impl Render {
    let router = use_router();
    let id = router.use_param("id");

    Effect::new(move |_| {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Enter {
            router.pop();
        }
    };

    view! {
        <paragraph v:id="child2" v:focusable on:key_down=key_down>
            {format!("child2 id={}", id.get().unwrap())}
        </paragraph>
    }
}
