use std::error::Error;

use rooibos::components::{self, CommandBuilder};
use rooibos::dom::Render;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(RuntimeSettings::default(), CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
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
