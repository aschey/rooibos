use std::process::ExitCode;

use modalkit::actions::Action;
use modalkit::editing::application::ApplicationAction;
use modalkit::editing::context::EditContext;
use modalkit::key::TerminalKey;
use modalkit::keybindings::SequenceStatus;
use rooibos::dom::{line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, col, mount, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos_keybind::{
    AppInfo, CommandBar, CommandCompleter, CommandGenerator, CommandHandler, KeyInputHandler,
    KeyMapper, extract, handle_command,
};
use rooibos_keybind_macros::{Commands, key};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    let mut cmd_handler = CommandHandler::<AppAction>::new();
    cmd_handler.add_commands::<AppAction>();

    mount(app);
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default().event_filter(move |event| cmd_handler.event_filter(event)),
        CrosstermBackend::stdout(),
    );
    runtime.run().await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let increase_count = move || set_count.update(|c| *c += 1);
    let decrease_count = move || set_count.update(|c| *c -= 1);

    let mut bindings = KeyMapper::new();
    bindings.map_action(&key!(<C-Up>), AppAction::Count(Direction::Up));
    bindings.map_action(&key!(<C-Down>), AppAction::Count(Direction::Down));

    bindings.map_handler(&key!(<Up>), move |_, _, _| increase_count());
    bindings.map_handler(&key!(<Down>), move |_, _, _| decrease_count());

    let key_handler = KeyInputHandler::new(bindings);

    handle_command(extract!(dir, AppAction::Count(dir)), move |dir| {
        if dir == Direction::Up {
            increase_count()
        } else {
            decrease_count()
        }
    });

    col![
        wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
            .on_key_down(key_handler)
            .on_click(move |_, _, _| increase_count()),
        CommandBar::<AppAction>::new().render()
    ]
}

#[derive(clap::Parser, Commands, Clone, Debug, PartialEq, Eq)]
pub enum AppAction {
    #[command(subcommand)]
    Count(Direction),
}

#[derive(clap::Subcommand, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}
