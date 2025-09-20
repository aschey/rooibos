use std::error::Error;
use std::io::{IsTerminal, stdin};
use std::process::ExitCode;

use rooibos::reactive::dom::{Render, line};
use rooibos::reactive::wgt;
use rooibos::runtime::Runtime;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, Box<dyn Error>>;

#[rooibos::main]
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

    let runtime = Runtime::initialize(DefaultBackend::auto().await?);
    Ok(runtime.run(|| app(input)).await?)
}

fn app(text: String) -> impl Render {
    wgt!(line!("You typed: ", text.clone().bold().green()))
}
