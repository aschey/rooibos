mod client;
mod server;

use std::error::Error;
use std::io::Stdout;

use client::{add_todo, fetch_todos};
use rooibos::dom::{col, row, suspense, widget_ref, Errors, KeyCode, Render, Suspend};
use rooibos::reactive::actions::Action;
use rooibos::reactive::computed::AsyncDerived;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::ArcRwSignal;
use rooibos::reactive::traits::{Get, Track, With};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime, RuntimeSettings};
use rooibos::tui::style::Stylize;
use rooibos::tui::text::{Line, Span};
use rooibos::tui::widgets::Paragraph;
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
    let todos = AsyncDerived::new(move || {
        // TODO: is this the best way to trigger a refetch?
        version.track();
        fetch_todos()
    });

    let keypress = use_keypress();

    Effect::new(move |_| {
        if let Some(keypress) = keypress.get() {
            if keypress.code == KeyCode::Char('a') {
                add_todo.dispatch("test".to_string());
            }
        }
    });

    row![col![suspense!(
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
}
