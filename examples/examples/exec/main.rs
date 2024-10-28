use std::env;
use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, mount, text};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, exec};
use rooibos::terminal::crossterm::CrosstermBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let editor = env::var("EDITOR").unwrap_or("vim".to_string());
    mount(|| app(editor, Vec::new()));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
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
