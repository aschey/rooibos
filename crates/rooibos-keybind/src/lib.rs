mod command_bar;

use std::collections::VecDeque;
use std::error::Error;
use std::io::empty;
use std::marker::PhantomData;
use std::sync::Arc;

pub use command_bar::*;
use modalkit::actions::{
    Action, CommandAction, CommandBarAction, Commandable, CursorAction, EditAction, EditorAction,
    PromptAction,
};
use modalkit::commands::{CommandMachine, CommandResult, CommandStep};
use modalkit::editing::application::{
    ApplicationAction, ApplicationContentId, ApplicationInfo, ApplicationStore, ApplicationWindowId,
};
use modalkit::editing::context::EditContext;
use modalkit::editing::key::KeyManager;
use modalkit::editing::store::Store;
use modalkit::env::CommonKeyClass;
use modalkit::env::vim::VimMode;
use modalkit::env::vim::command::{
    CommandContext, CommandDescription, VimCommand, VimCommandMachine,
};
use modalkit::env::vim::keybindings::{InputStep, VimBindings, VimMachine};
use modalkit::errors::EditResult;
use modalkit::key::TerminalKey;
use modalkit::keybindings::{
    BindingMachine, EdgeEvent, EdgeRepeat, InputBindings, InputKey, ModalMachine, SequenceStatus,
};
use modalkit::prelude::{
    CommandType, Count, EditTarget, MoveDir1D, MoveType, RepeatType, Specifier,
};
use rooibos_dom::KeyHandler;
use rooibos_reactive::graph::owner::{StoredValue, provide_context, use_context};
use terminput::{Event, KeyEvent};
use wasm_compat::sync::{Mutex, RwLock};

pub struct KeyInputHandler<A, S>
where
    A: ApplicationInfo,
{
    manager: KeyManager<TerminalKey, Action<A>, S>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppInfo<T> {
    _phantom: PhantomData<T>,
}

impl<T> Default for AppInfo<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AppInfo<T> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum AppAction {
//     DoTheThing,
// }

// impl From<AppAction> for Action<AppInfo> {
//     fn from(value: AppAction) -> Self {
//         Action::Application(value)
//     }
// }

pub struct AppStore {}

impl ApplicationStore for AppStore {}

// impl ApplicationAction for AppAction {
//     fn is_edit_sequence(
//         &self,
//         ctx: &modalkit::editing::context::EditContext,
//     ) -> modalkit::keybindings::SequenceStatus {
//         SequenceStatus::Break
//     }
//
//     fn is_last_action(
//         &self,
//         ctx: &modalkit::editing::context::EditContext,
//     ) -> modalkit::keybindings::SequenceStatus {
//         SequenceStatus::Atom
//     }
//
//     fn is_last_selection(
//         &self,
//         ctx: &modalkit::editing::context::EditContext,
//     ) -> modalkit::keybindings::SequenceStatus {
//         SequenceStatus::Ignore
//     }
//
//     fn is_switchable(&self, ctx: &EditContext) -> bool {
//         false
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppId {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppContentId {
    Command(CommandType),
}

impl ApplicationContentId for AppContentId {}

impl ApplicationWindowId for AppId {}

impl<T> ApplicationInfo for AppInfo<T>
where
    T: ApplicationAction,
{
    type Error = String;

    type Action = T;

    type Store = AppStore;

    type WindowId = AppId;

    type ContentId = AppContentId;

    fn content_of_command(cmdtype: CommandType) -> Self::ContentId {
        AppContentId::Command(cmdtype)
    }
}

impl<A, S> KeyInputHandler<A, S>
where
    A: ApplicationInfo,
{
    pub fn new(
        bindings: impl BindingMachine<TerminalKey, Action<A>, S, EditContext> + Send + Sync + 'static,
    ) -> Self {
        Self {
            manager: KeyManager::new(bindings),
        }
    }

    pub fn read(&mut self, key_event: KeyEvent) {
        let key_event: Result<crossterm::event::KeyEvent, _> = key_event.try_into();
        let Ok(key_event) = key_event else {
            return;
        };

        self.manager.input_key(key_event.into());

        while let Some((action, context)) = self.manager.pop() {
            println!("{action:?}");
            // match action {
            //     Action::NoOp => todo!(),
            //     Action::Editor(_) => todo!(),
            //     Action::Macro(_) => todo!(),
            //     Action::Jump(_, _, _) => todo!(),
            //     Action::Repeat(_) => todo!(),
            //     Action::Scroll(_) => todo!(),
            //     Action::KeywordLookup => todo!(),
            //     Action::RedrawScreen => todo!(),
            //     Action::ShowInfoMessage(_) => todo!(),
            //     Action::Suspend => todo!(),
            //     Action::Search(_, _) => todo!(),
            //     Action::Command(_) => todo!(),
            //     Action::CommandBar(_) => todo!(),
            //     Action::Prompt(_) => todo!(),
            //     Action::Tab(_) => todo!(),
            //     Action::Window(_) => todo!(),
            //     Action::Application(_) => todo!(),
            //     _ => todo!(),
            // }
        }
    }
}

impl<A, S> KeyHandler for KeyInputHandler<A, S>
where
    A: ApplicationInfo,
{
    fn handle(
        &mut self,
        event: KeyEvent,
        data: rooibos_dom::EventData,
        handle: rooibos_dom::EventHandle,
    ) {
        self.read(event)
    }
}

pub fn once(key: &TerminalKey) -> (EdgeRepeat, EdgeEvent<TerminalKey, CommonKeyClass>) {
    (EdgeRepeat::Once, EdgeEvent::Key(*key))
}

pub struct CommandHandler<T>
where
    T: ApplicationAction,
{
    manager: KeyManager<TerminalKey, Action<AppInfo<T>>, RepeatType>,
    cmds: CommandMachine<VimCommand<AppInfo<T>>>,
    context: CommandBarContext<T>,
    command_focused: bool,
    action_stack: VecDeque<(Action<AppInfo<T>>, EditContext)>,
}

impl<T> Default for CommandHandler<T>
where
    T: ApplicationAction + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct CommandBarContext<T>
where
    T: ApplicationAction,
{
    store: StoredValue<Store<AppInfo<T>>>,
    action_handlers: Arc<
        RwLock<
            Vec<
                Box<
                    dyn Fn(
                            CommandHandlerAction<T>,
                            &EditContext,
                        ) -> Vec<(Action<AppInfo<T>>, EditContext)>
                        + Send
                        + Sync,
                >,
            >,
        >,
    >,
}

impl<T> CommandBarContext<T>
where
    T: ApplicationAction,
{
    pub fn on_command_bar_action<F>(&self, f: F)
    where
        F: Fn(CommandHandlerAction<T>, &EditContext) -> Vec<(Action<AppInfo<T>>, EditContext)>
            + Send
            + Sync
            + 'static,
    {
        let mut handlers = self.action_handlers.write();
        handlers.push(Box::new(f))
    }

    pub fn store(&self) -> StoredValue<Store<AppInfo<T>>> {
        self.store
    }
}

pub fn use_command_context<T>() -> CommandBarContext<T>
where
    T: ApplicationAction + Send + Sync + 'static,
{
    use_context::<CommandBarContext<T>>().unwrap()
}

pub fn provide_command_context<T>()
where
    T: ApplicationAction + Send + Sync + 'static,
{
    provide_context(CommandBarContext {
        store: StoredValue::new(Store::<AppInfo<T>>::new(AppStore {})),
        action_handlers: Arc::new(RwLock::new(Vec::new())),
    });
}

pub enum CommandHandlerAction<T>
where
    T: ApplicationAction,
{
    CommandBar(CommandBarAction<AppInfo<T>>),
    Editor(EditorAction),
    Prompt(PromptAction),
}

impl<T> CommandHandler<T>
where
    T: ApplicationAction + Send + Sync + 'static,
{
    pub fn new() -> Self {
        let mut ism = ModalMachine::<TerminalKey, InputStep<AppInfo<T>>>::empty();
        let colon = ":".parse::<TerminalKey>().unwrap();
        let esc = "<Esc>".parse::<TerminalKey>().unwrap();
        let enter = "<Enter>".parse::<TerminalKey>().unwrap();
        let up = "<Up>".parse::<TerminalKey>().unwrap();
        let down = "<Down>".parse::<TerminalKey>().unwrap();
        let left = "<Left>".parse::<TerminalKey>().unwrap();
        let right = "<Right>".parse::<TerminalKey>().unwrap();

        ism.add_mapping(
            VimMode::Normal,
            &[once(&colon)],
            &InputStep::<AppInfo<T>>::new()
                .actions(vec![Action::CommandBar(CommandBarAction::Focus(
                    ":".into(),
                    CommandType::Command,
                    Box::new(Action::Command(CommandAction::Execute(Count::Exact(1)))),
                ))])
                .goto(VimMode::Command),
        );

        ism.add_mapping(
            VimMode::Command,
            &[once(&esc)],
            &InputStep::<AppInfo<T>>::new()
                .actions(vec![Action::CommandBar(CommandBarAction::Unfocus)])
                .goto(VimMode::Normal),
        );
        ism.add_mapping(
            VimMode::Command,
            &[once(&enter)],
            &InputStep::<AppInfo<T>>::new()
                .actions(vec![Action::Prompt(PromptAction::Submit)])
                .goto(VimMode::Normal),
        );
        ism.add_mapping(
            VimMode::Command,
            &[once(&up)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Prompt(PromptAction::Recall(
                MoveDir1D::Previous,
                Count::Contextual,
                true,
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &[once(&down)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Prompt(PromptAction::Recall(
                MoveDir1D::Next,
                Count::Contextual,
                true,
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &[once(&left)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Editor(EditorAction::Edit(
                Specifier::Contextual,
                EditTarget::Motion(
                    MoveType::Column(MoveDir1D::Previous, true),
                    Count::Contextual,
                ),
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &[once(&right)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Editor(EditorAction::Edit(
                Specifier::Contextual,
                EditTarget::Motion(MoveType::Column(MoveDir1D::Next, true), Count::Contextual),
            ))]),
        );
        ism.get_cursor_indicator();

        let cmds = VimCommandMachine::<AppInfo<T>>::default();
        // cmds.add_command(vimcommand::<appinfo> {
        //     name: "do".into(),
        //     aliases: vec![],
        //     f: handler,
        // });

        Self {
            manager: KeyManager::new(ism),
            context: use_context::<CommandBarContext<T>>().unwrap(),
            command_focused: false,
            action_stack: VecDeque::new(),
            cmds,
        }
    }

    pub fn add_command(&mut self, cmd: VimCommand<AppInfo<T>>) {
        self.cmds.add_command(cmd);
    }

    fn action_pop(&mut self, keyskip: bool) -> Option<(Action<AppInfo<T>>, EditContext)> {
        if let res @ Some(_) = self.action_stack.pop_front() {
            return res;
        }

        if keyskip { None } else { self.manager.pop() }
    }

    fn action_prepend(&mut self, acts: Vec<(Action<AppInfo<T>>, EditContext)>) {
        let mut acts = VecDeque::from(acts);
        acts.append(&mut self.action_stack);
        self.action_stack = acts;
    }

    pub fn event_filter(&mut self, event: Event) -> Option<Event> {
        let Event::Key(key_event) = event else {
            return Some(event);
        };
        let key_event: Result<crossterm::event::KeyEvent, _> = key_event.try_into();
        let Ok(key_event) = key_event else {
            return Some(event);
        };

        self.manager.input_key(key_event.into());
        let mut handled = false;

        while let Some((action, context)) = self.action_pop(false) {
            let mut new_actions = Vec::new();

            match action {
                Action::CommandBar(command_bar_action) => {
                    let handlers = self.context.action_handlers.read();
                    for handler in handlers.iter() {
                        let actions = (handler)(
                            CommandHandlerAction::CommandBar(command_bar_action.clone()),
                            &context,
                        );

                        new_actions.push(actions);
                    }
                    self.command_focused =
                        matches!(command_bar_action, CommandBarAction::Focus(_, _, _));
                    handled = true;
                }
                Action::Command(command_action) => {
                    let mut store = self.context.store.write_value();
                    let actions = self
                        .cmds
                        .command(&command_action, &context, &mut store.registers)
                        .unwrap();
                    new_actions.push(actions);
                }
                Action::Editor(editor_action) if self.command_focused => {
                    let handlers = self.context.action_handlers.read();
                    for handler in handlers.iter() {
                        let actions = (handler)(
                            CommandHandlerAction::Editor(editor_action.clone()),
                            &context,
                        );
                        new_actions.push(actions);
                    }
                    handled = true;
                }
                Action::Prompt(prompt_action) if self.command_focused => {
                    let handlers = self.context.action_handlers.read();
                    for handler in handlers.iter() {
                        let actions = (handler)(
                            CommandHandlerAction::Prompt(prompt_action.clone()),
                            &context,
                        );
                        new_actions.push(actions);
                    }
                    handled = true;
                }
                Action::Application(app_action) => {}

                act => {
                    //println!("{act:?}");
                }
            }
            for new_action in new_actions {
                self.action_prepend(new_action);
            }
        }

        if handled { None } else { Some(event) }
    }
}
