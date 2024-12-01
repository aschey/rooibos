use std::env;
use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::{chars, shrink};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, mount, text};
use rooibos::reactive::{col, row};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, exec};
use rooibos::terminal::crossterm::CrosstermBackend;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let editor = env::var("EDITOR").unwrap_or("vim".to_string());
    Runtime::initialize(CrosstermBackend::stdout())
        .run(|| app(editor, Vec::new()))
        .await
}

fn app(editor: String, args: Vec<String>) -> impl Render {
    row![col![
        Button::new()
            .on_click(move || {
                let mut cmd = tokio::process::Command::new(&editor);
                cmd.args(&args);
                exec(cmd, |_, _, _| {});
            })
            .render(text!("Open Editor"))
    ]]
}

#[cfg(test)]
mod tests;
