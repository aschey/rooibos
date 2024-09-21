use rooibos::dom::{KeyCode, KeyEvent, focus_id};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, after_render, col, mount, wgt};
use rooibos::router::{DefaultRoute, Route, RouteFromStatic, Router, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Paragraph;
type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Route)]
struct Child1 {
    id: Option<u32>,
}

impl Child1 {
    fn new(id: Option<u32>) -> Self {
        Self { id }
    }
}

#[derive(Route)]
struct Child2 {
    id: i32,
}

impl Child2 {
    fn new(id: i32) -> Self {
        Self { id }
    }
}

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let child2_id = RwSignal::new(0);

    col![Router::new().routes([
        Route::new::<DefaultRoute>(child0),
        Route::new::<Child1>(move || child1(child2_id)),
        Route::new::<Child2>(child2)
    ])]
}

fn child0() -> impl Render {
    let router = use_router();

    Effect::new(move || {
        focus_id("child0");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Child1::new(Some(1)));
        }
    };

    wgt!(Paragraph::new("child0"))
        .on_key_down(key_down)
        .id("child0")
}

fn child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.use_query(Child1::ID);

    after_render(move || {
        focus_id("child1");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.push(Child2::new(child2_id.get()));
            child2_id.update(|id| *id += 1);
        }
    };

    wgt!(Paragraph::new(format!("child1 id={}", id.get())))
        .on_key_down(key_down)
        .id("child1")
}

fn child2() -> impl Render {
    let router = use_router();
    let id = router.use_param(Child2::ID);

    after_render(move || {
        focus_id("child2");
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            router.pop();
        }
    };

    wgt!(Paragraph::new(format!("child2 id={}", id.get())))
        .on_key_down(key_down)
        .id("child2")
}
