use reqwest::Client;

use crate::server::{CreateTodo, Todo, UpdateTodo};

pub async fn fetch_todos() -> rooibos::reactive::error::Result<Vec<Todo>> {
    let res = Client::new()
        .get("http://localhost:9353/todos")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Todo>>()
        .await?;

    Ok(res)
}

pub async fn add_todo(text: String) -> rooibos::reactive::error::Result<()> {
    Client::new()
        .post("http://localhost:9353/todos")
        .json(&CreateTodo { text })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn update_todo(id: u32, text: String) -> rooibos::reactive::error::Result<()> {
    Client::new()
        .patch(format!("http://localhost:9353/todos/{id}"))
        .json(&UpdateTodo {
            text: Some(text),
            completed: None,
        })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn delete_todo(id: u32) -> rooibos::reactive::error::Result<()> {
    Client::new()
        .delete(format!("http://localhost:9353/todos/{id}"))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
