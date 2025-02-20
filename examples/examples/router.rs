use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::{Borders, align_items, borders, chars};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, text};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, height, row, wgt, width};
use rooibos::router::{Route, RouteFromStatic, Router, provide_router, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use taffy::AlignItems;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[derive(Route)]
struct Home;

#[derive(Route)]
struct About;

#[derive(Route)]
struct BlogIndex;

#[derive(Route)]
struct BlogPost {
    id: usize,
}

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    provide_router();
    col![
        props(align_items(AlignItems::Center), width!(30.),),
        col![
            props(height!(10.), borders(Borders::all())),
            Router::new()
                .routes([
                    Route::new::<Home>(home),
                    Route::new::<About>(about),
                    Route::new::<BlogIndex>(blog_index),
                    Route::new::<BlogPost>(blog_post)
                ])
                .initial(Home)
        ],
        footer()
    ]
}

fn home() -> impl Render {
    let router = use_router();
    let about_click = move || router.push(About);
    let blog_click = move || router.push(BlogIndex);
    col![
        props(align_items(AlignItems::Center)),
        wgt!(props(width!(22.), height!(2.)), "This is the home page"),
        row![
            props(width!(18.)),
            Button::new()
                .width(chars(9.))
                .height(chars(3.))
                .on_click(about_click)
                .render(text!("About")),
            Button::new()
                .width(chars(9.))
                .height(chars(3.))
                .on_click(blog_click)
                .render(text!("Blog"))
        ]
    ]
}

fn about() -> impl Render {
    wgt!("This is the about page")
}

fn blog_index() -> impl Render {
    let router = use_router();
    let route_to_post = move |id: usize| router.push(BlogPost { id });
    col![
        wgt!("This is the blog page"),
        Button::new()
            .height(chars(3.))
            .on_click(move || route_to_post(1))
            .render(text!("post 1")),
        Button::new()
            .height(chars(3.))
            .on_click(move || route_to_post(2))
            .render(text!("post 2"))
    ]
}

fn blog_post() -> impl Render {
    let router = use_router();
    let id = router.use_param(BlogPost::ID);
    wgt!(format!("blog post {}", id.get()))
}

fn footer() -> impl Render {
    let router = use_router();
    let on_forward = move || router.forward();
    let on_back = move || router.back();
    row![
        props(width!(10.), height!(3.)),
        Button::new()
            .height(chars(3.))
            .on_click(on_back)
            .enabled(router.can_go_back())
            .render(text!("←")),
        Button::new()
            .height(chars(3.))
            .on_click(on_forward)
            .enabled(router.can_go_forward())
            .render(text!("→"))
    ]
}
