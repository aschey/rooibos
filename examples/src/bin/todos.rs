use std::collections::HashMap;
use std::error::Error;
use std::io::Stdout;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use axum::Json;
use reqwest::Client;
use rooibos::dom::{
    col, error_boundary, row, suspense, widget_ref, Errors, KeyCode, Render, Suspend,
};
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
use serde::{Deserialize, Serialize};

static ID: AtomicU32 = AtomicU32::new(2);

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    tokio::spawn(server());
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
        // TODO: seems a bit hacky, probably a better way to trigger a refetch
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

    row![col![suspense(
        move || widget_ref!(Line::from(" Loading...".gray())),
        move || error_boundary(
            move || {
                Suspend(async move {
                    todos.await.map(|todos| {
                        widget_ref!(Paragraph::new(
                            todos
                                .iter()
                                .map(|t| Line::from(t.text.clone()))
                                .collect::<Vec<_>>()
                        ))
                    })
                })
            },
            fallback
        )
    )]]
}

async fn fetch_todos() -> rooibos::dom::Result<Vec<Todo>> {
    let res = Client::new()
        .get("http://localhost:9353/todos")
        .send()
        .await?
        .json::<Vec<Todo>>()
        .await?;
    Ok(res)
}

async fn add_todo(text: String) -> rooibos::dom::Result<()> {
    Client::new()
        .post("http://localhost:9353/todos")
        .json(&CreateTodo { text })
        .send()
        .await?;

    Ok(())
}

async fn server() {
    let db = Db::default();
    db.write().unwrap().insert(
        1,
        Todo {
            id: 1,
            text: "test".to_owned(),
            completed: false,
        },
    );

    let app = axum::Router::new()
        .route("/todos", get(todos_get).post(todos_create))
        .route("/todos/:id", patch(todos_update).delete(todos_delete))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn todos_get(State(db): State<Db>) -> impl IntoResponse {
    let todos = db.read().unwrap();

    let todos = todos.values().cloned().collect::<Vec<_>>();

    Json(todos)
}

async fn todos_create(State(db): State<Db>, Json(input): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo {
        id: ID.fetch_add(1, Ordering::SeqCst),
        text: input.text,
        completed: false,
    };

    db.write().unwrap().insert(todo.id, todo.clone());

    (StatusCode::CREATED, Json(todo))
}

async fn todos_update(
    Path(id): Path<u32>,
    State(db): State<Db>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    db.write().unwrap().insert(todo.id, todo.clone());

    Ok(Json(todo))
}

async fn todos_delete(Path(id): Path<u32>, State(db): State<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTodo {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

type Db = Arc<RwLock<HashMap<u32, Todo>>>;
