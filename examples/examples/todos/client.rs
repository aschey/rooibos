use reqwest::Client;

use crate::server::{CreateTodo, Todo};

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
