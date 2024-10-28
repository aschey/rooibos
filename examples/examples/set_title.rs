use std::io::Stdout;
use std::process::ExitCode;

use rooibos::keybind::map_handler;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, mount, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, set_title};
use rooibos::terminal::crossterm::{CrosstermBackend, TerminalSettings};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::<Stdout>::new(
        TerminalSettings::default().title("initial title"),
    ));
    runtime.run().await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    Effect::new(move |prev: Option<()>| {
        let count = count.get();
        if prev.is_some() {
            set_title(format!("count {count}")).unwrap();
        }
    });

    wgt!(format!("count {}", count.get()))
        .on_key_down(map_handler("<Enter>", move |_, _| {
            update_count();
        }))
        .on_click(move |_| update_count())
}
