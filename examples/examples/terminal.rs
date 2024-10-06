use std::process::ExitCode;

use rooibos::components::{self, CommandBuilder};
use rooibos::reactive::{Render, mount};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::Block;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let term_ref = components::Terminal::get_ref();
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new_default_prog();
    cmd.cwd(cwd);
    term_ref.spawn_command(cmd);

    components::Terminal::default()
        .block(Block::bordered().title("Terminal"))
        .render(term_ref)
}
