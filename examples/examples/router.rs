use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::{Borders, align_items, borders, chars};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, text};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, height, row, wgt, width};
use rooibos::router::{Route, RouteContext, RouteFromStatic, use_router};
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
    let (router, route_context) = use_router();

    col![
        props(align_items(AlignItems::Center), width!(30.),),
        col![
            props(height!(10.), borders(Borders::all())),
            router
                .routes([
                    Route::new::<Home>(move || home(route_context)),
                    Route::new::<About>(about),
                    Route::new::<BlogIndex>(move || blog_index(route_context)),
                    Route::new::<BlogPost>(move || blog_post(route_context)),
                ])
                .initial(Home)
                .render()
        ],
        footer(route_context)
    ]
}

fn home(route_context: RouteContext) -> impl Render {
    let about_click = move || route_context.push(About);
    let blog_click = move || route_context.push(BlogIndex);
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

fn blog_index(route_context: RouteContext) -> impl Render {
    let route_to_post = move |id: usize| route_context.push(BlogPost { id });
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

fn blog_post(route_context: RouteContext) -> impl Render {
    let id = route_context.use_param(BlogPost::ID);
    wgt!(format!("blog post {}", id.get()))
}

fn footer(route_context: RouteContext) -> impl Render {
    let on_forward = move || route_context.forward();
    let on_back = move || route_context.back();
    row![
        props(width!(10.), height!(3.)),
        Button::new()
            .height(chars(3.))
            .on_click(on_back)
            .enabled(route_context.can_go_back())
            .render(text!("←")),
        Button::new()
            .height(chars(3.))
            .on_click(on_forward)
            .enabled(route_context.can_go_forward())
            .render(text!("→"))
    ]
}
