use std::process::ExitCode;

use modalkit::actions::Action;
use modalkit::commands::{CommandResult, CommandStep};
use modalkit::editing::application::ApplicationAction;
use modalkit::editing::context::EditContext;
use modalkit::env::vim::command::{CommandContext, CommandDescription, VimCommand};
use modalkit::keybindings::SequenceStatus;
use rooibos::dom::{KeyCode, KeyEvent, line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, col, mount, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos_keybind::{AppInfo, CommandBar, CommandHandler, provide_command_context};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    provide_command_context::<AppAction>();
    let mut cmd_handler = CommandHandler::<AppAction>::new();
    cmd_handler.add_command(VimCommand::<AppInfo<AppAction>> {
        name: "do".into(),
        aliases: vec![],
        f: handler,
    });

    mount(app);
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default().event_filter(move |event| cmd_handler.event_filter(event)),
        CrosstermBackend::stdout(),
    );
    runtime.run().await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };
    col![
        wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
            .on_key_down(key_down)
            .on_click(move |_, _, _| update_count()),
        CommandBar::<AppAction>::new().render()
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppAction {
    DoTheThing,
}

impl From<AppAction> for Action<AppInfo<AppAction>> {
    fn from(value: AppAction) -> Self {
        Action::Application(value)
    }
}

impl ApplicationAction for AppAction {
    fn is_edit_sequence(
        &self,
        ctx: &modalkit::editing::context::EditContext,
    ) -> modalkit::keybindings::SequenceStatus {
        SequenceStatus::Break
    }

    fn is_last_action(
        &self,
        ctx: &modalkit::editing::context::EditContext,
    ) -> modalkit::keybindings::SequenceStatus {
        SequenceStatus::Atom
    }

    fn is_last_selection(
        &self,
        ctx: &modalkit::editing::context::EditContext,
    ) -> modalkit::keybindings::SequenceStatus {
        SequenceStatus::Ignore
    }

    fn is_switchable(&self, ctx: &EditContext) -> bool {
        false
    }
}

fn handler(
    desc: CommandDescription,
    ctx: &mut CommandContext,
) -> CommandResult<VimCommand<AppInfo<AppAction>>> {
    let act = AppAction::DoTheThing;
    println!("yoo {desc:?}");
    Ok(CommandStep::Continue(act.into(), ctx.context.clone()))
}
