use std::env;

use rooibos::components::Button;
use rooibos::dom::layout::chars;
use rooibos::dom::{text, Render, UpdateLayoutProps};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{exec, Runtime};

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or("vim".to_string());

    let runtime = Runtime::initialize(CrosstermBackend::stdout(), || app(editor, Vec::new()));
    runtime.run().await?;
    Ok(())
}

fn app(editor: String, args: Vec<String>) -> impl Render {
    Button::new()
        .width(chars(20.))
        .height(chars(3.))
        .on_click(move || {
            let mut cmd = tokio::process::Command::new(&editor);
            cmd.args(&args);
            exec(cmd, |_, _, _| {});
        })
        .render(text!("Open Editor"))
}

#[cfg(test)]
mod tests;
