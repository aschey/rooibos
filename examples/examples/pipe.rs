use std::error::Error;
use std::io::{stdin, IsTerminal};

use rooibos::reactive::{line, wgt, Render};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
use rooibos::tui::style::Stylize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let input = {
        let input = stdin();
        if input.is_terminal() {
            return Err("Pipe in some text")?;
        }

        let mut buffer = String::new();
        input.read_line(&mut buffer)?;
        buffer
    };

    let runtime = Runtime::initialize(CrosstermBackend::stdout(), || app(input));
    runtime.run().await?;
    Ok(())
}

fn app(text: String) -> impl Render {
    wgt!(line!("You typed: ", text.clone().bold().green()))
}
