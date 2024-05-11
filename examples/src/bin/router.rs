use std::error::Error;

use rooibos::components::{use_router, Route, Router};
use rooibos::dom::{col, focus_id, focus_next, mount, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::runtime::run;
use rooibos::tui::widgets::Paragraph;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    run().await?;
    Ok(())
}

fn app() -> impl Render {
    let child2_id = RwSignal::new(0);

    Effect::new(move |_| {
        focus_next();
    });

    col![Router::new().routes([
        Route::new("/", child0),
        Route::new("/child1", move || child1(child2_id)),
        Route::new("/child2/{id}", child2)
    ])]
}

fn child0() -> impl Render {
    let router = use_router();

    Effect::new(move |_| {
        focus_id("child0");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push("/child1?id=1");
        }
    };

    widget_ref!(Paragraph::new("child0"))
        .focusable(true)
        .on_key_down(key_down)
        .id("child0")
}

fn child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.use_query("id");

    Effect::new(move |_| {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.push(format!("/child2/{}", child2_id.get_untracked()));
            child2_id.update(|id| *id += 1);
        }
    };

    widget_ref!(Paragraph::new(format!("child1 id={}", id.get().unwrap())))
        .focusable(true)
        .on_key_down(key_down)
        .id("child1")
}

fn child2() -> impl Render {
    let router = use_router();
    let id = router.use_param("id");

    Effect::new(move |_| {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            router.pop();
        }
    };

    widget_ref!(Paragraph::new(format!("child2 id={}", id.get().unwrap())))
        .focusable(true)
        .on_key_down(key_down)
        .id("child2")
}
