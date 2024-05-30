mod client;
mod server;

use std::error::Error;
use std::io::Stdout;

use client::{add_todo, fetch_todos};
use rooibos::components::{Input, Popup};
use rooibos::dom::{
    clear, col, overlay, row, transition, widget_ref, Constrainable, Errors, Render, Suspend,
};
use rooibos::reactive::actions::Action;
use rooibos::reactive::computed::AsyncDerived;
use rooibos::reactive::signal::ArcRwSignal;
use rooibos::reactive::traits::{Get, Track, With};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::style::Stylize;
use rooibos::tui::text::{Line, Span};
use rooibos::tui::widgets::{Block, Paragraph};
use server::run_server;

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    tokio::spawn(run_server());
    runtime.run().await?;

    Ok(())
}

fn app() -> impl Render {
    let fallback = move |errors: ArcRwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| Span::from(e.to_string()))
                    .collect::<Vec<_>>()
            })
        };

        widget_ref!(Paragraph::new(Line::from(error_list())))
    };

    let add_todo = Action::new(move |text: &String| add_todo(text.clone()));
    let version = add_todo.version();
    let add_pending = add_todo.pending();
    let todos = AsyncDerived::new(move || {
        // TODO: is this the best way to trigger a refetch?
        version.track();
        fetch_todos()
    });

    let input_ref = Input::get_ref();

    overlay![
        col![
            row![
                col![widget_ref!("Add a Todo")]
                    .block(Block::default())
                    .length(12),
                col![
                    Input::default()
                        .block(Block::bordered())
                        .on_submit(move |val| {
                            add_todo.dispatch(val);
                            input_ref.delete_line_by_head();
                        })
                        .length(3)
                        .render(input_ref)
                ]
            ]
            .length(3),
            row![col![transition!(
                widget_ref!(Line::from(" Loading...".gray())),
                todos.await.map(|todos| {
                    widget_ref!(Paragraph::new(
                        todos
                            .iter()
                            .map(|t| Line::from(t.text.clone()))
                            .collect::<Vec<_>>()
                    ))
                }),
                fallback
            )]]
            .block(Block::bordered().title("Todos"))
        ],
        Popup::default().percent_x(50).percent_y(50).render(
            move || add_pending.get(),
            move || col![
                col![].fill(1),
                clear![widget_ref!(
                    Paragraph::new("Saving...").block(Block::bordered())
                )]
                .length(3),
                col![].fill(1),
            ]
        )
    ]
}
