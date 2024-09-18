use std::env;

use rooibos::components::Button;
use rooibos::dom::text;
use rooibos::reactive::layout::chars;
use rooibos::reactive::{mount, Render, UpdateLayoutProps};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{exec, Runtime};
use rooibos::terminal::crossterm::CrosstermBackend;

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or("vim".to_string());
    mount(|| app(editor, Vec::new()));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
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
