use std::process::ExitCode;

use rooibos::dom::{line, span};
use rooibos::keybind::{
    CommandBar, CommandFilter, CommandHandler, Commands, KeyInputHandler, extract, handle_command,
    map_action, map_handler, parse,
};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::layout::chars;
use rooibos::reactive::{Render, UpdateLayoutProps, col, mount, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    let mut cmd_handler = CommandHandler::<AppAction>::new();
    cmd_handler.generate_commands();

    mount(app);
    let runtime = Runtime::initialize_with(
        RuntimeSettings::default().handle_commands(cmd_handler),
        CrosstermBackend::stdout(),
    );
    runtime.run().await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let increase_count = move || set_count.update(|c| *c += 1);
    let decrease_count = move || set_count.update(|c| *c -= 1);

    let key_handler = KeyInputHandler::<AppAction>::new([
        map_action(parse("<C-Up>"), AppAction::Count { val: 1 }),
        map_action(parse("<C-Down>"), AppAction::Count { val: -1 }),
        map_handler(parse("<Up>"), move |_| increase_count()),
        map_handler(parse("<Down>"), move |_| decrease_count()),
    ]);

    handle_command(extract!(val, AppAction::Count { val }), move |val| {
        set_count.update(|c| *c += val);
    });

    col![
        wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
            .on_key_down(key_handler)
            .on_click(move |_, _, _| increase_count())
            .grow(1.),
        CommandBar::<AppAction>::new().height(chars(1.)).render()
    ]
}

#[derive(clap::Parser, Commands, Clone, Debug, PartialEq, Eq)]
pub enum AppAction {
    Count {
        #[arg(allow_hyphen_values(true))]
        val: i32,
    },
}
