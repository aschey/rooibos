use std::process::ExitCode;

use rooibos::keybind::{
    Bind, CommandBar, CommandFilter, CommandHandler, Commands, KeyActionMap, extract, keys,
    on_command,
};
use rooibos::reactive::dom::layout::{full, height, width};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let mut cmd_handler = CommandHandler::<AppAction>::new();
    cmd_handler.generate_commands();

    let runtime = Runtime::initialize_with(
        RuntimeSettings::default().handle_commands(cmd_handler),
        DefaultBackend::auto().await?,
    );
    runtime.run(app).await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let increase_count = move || set_count.update(|c| *c += 1);
    let decrease_count = move || set_count.update(|c| *c -= 1);

    on_command(extract!(val, AppAction::Count { val }), move |val| {
        set_count.update(|c| *c += val);
    });

    col![
        style(width(full()), height(full())),
        wgt!(line!("count: ".bold(), count.get().cyan())).on_key_down(
            [
                KeyActionMap::action(
                    keys::combine([keys::CTRL, keys::UP]),
                    AppAction::Count { val: 1 }
                ),
                KeyActionMap::action(
                    keys::combine([keys::CTRL, keys::DOWN]),
                    AppAction::Count { val: -1 }
                ),
                KeyActionMap::handler(keys::UP, move |_, _| increase_count()),
                KeyActionMap::handler(keys::DOWN, move |_, _| decrease_count()),
            ]
            .bind()
        ),
        CommandBar::<AppAction>::new().height(1).render()
    ]
    .on_click(move |_| increase_count())
}

#[derive(clap::Parser, Commands, Clone, Debug, PartialEq, Eq)]
pub enum AppAction {
    Count {
        #[arg(allow_hyphen_values(true))]
        val: i32,
    },
}
