use reqwest::Client;

use crate::server::{CreateTodo, Todo, UpdateTodo};

pub async fn fetch_todos() -> rooibos::dom::Result<Vec<Todo>> {
    let res = Client::new()
        .get("http://localhost:9353/todos")
        .send()
        .await?
        .json::<Vec<Todo>>()
        .await?;
    Ok(res)
}

pub async fn add_todo(text: String) -> rooibos::dom::Result<()> {
    Client::new()
        .post("http://localhost:9353/todos")
        .json(&CreateTodo { text })
        .send()
        .await?;

    Ok(())
}

pub async fn update_todo(id: u32, text: String) -> rooibos::dom::Result<()> {
    Client::new()
        .patch(format!("http://localhost:9353/todos/{id}"))
        .json(&UpdateTodo {
            text: Some(text),
            completed: None,
        })
        .send()
        .await?;

    Ok(())
}
pub async fn delete_todo(id: u32) -> rooibos::dom::Result<()> {
    Client::new()
        .delete(format!("http://localhost:9353/todos/{id}"))
        .send()
        .await?;

    Ok(())
}
