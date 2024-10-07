use std::error::Error;
use std::io::{IsTerminal, stdin};
use std::process::ExitCode;

use rooibos::dom::line;
use rooibos::reactive::{Render, mount, wgt};
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, Box<dyn Error>>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    let input = {
        let input = stdin();
        if input.is_terminal() {
            return Err("Pipe in some text. Ex: echo hi | cargo run --example=pipe_input")?;
        }

        let mut buffer = String::new();
        while input.read_line(&mut buffer)? > 0 {}
        buffer
    };

    mount(|| app(input));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    Ok(runtime.run().await?)
}

fn app(text: String) -> impl Render {
    wgt!(line!("You typed: ", text.clone().bold().green()))
}
