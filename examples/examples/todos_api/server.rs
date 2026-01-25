use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use serde::{Deserialize, Serialize};

static ID: AtomicU32 = AtomicU32::new(1);

pub async fn run_server(listener: tokio::net::TcpListener) {
    let db = Db::default();

    let app = axum::Router::new()
        .route("/todos", get(todos_get).post(todos_create))
        .route("/todos/{id}", patch(todos_update).delete(todos_delete))
        .with_state(db);

    axum::serve(listener, app).await.unwrap();
}

async fn todos_get(State(db): State<Db>) -> Result<impl IntoResponse, StatusCode> {
    let todos = db.read().unwrap();

    let mut todos = todos.values().cloned().collect::<Vec<_>>();
    todos.sort_by_key(|t| t.id);

    Ok(Json(todos))
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
pub struct CreateTodo {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub text: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

type Db = Arc<RwLock<HashMap<u32, Todo>>>;
