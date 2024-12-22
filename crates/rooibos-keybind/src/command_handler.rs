use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::marker::PhantomData;
use std::sync::Arc;

use clap::Parser;
use educe::Educe;
use modalkit::actions::{
    Action, CommandAction, CommandBarAction, Commandable, EditAction, EditorAction, PromptAction,
};
use modalkit::commands::{CommandMachine, CommandResult, CommandStep, ParsedCommand};
use modalkit::editing::application::{
    ApplicationAction, ApplicationContentId, ApplicationInfo, ApplicationStore, ApplicationWindowId,
};
use modalkit::editing::context::EditContext;
use modalkit::editing::cursor::Cursor;
use modalkit::editing::key::KeyManager;
use modalkit::editing::rope::EditRope;
use modalkit::editing::store::Store;
use modalkit::env::vim::VimMode;
use modalkit::env::vim::command::{
    CommandContext, CommandDescription, VimCommand, VimCommandMachine,
};
use modalkit::env::vim::keybindings::InputStep;
use modalkit::key::TerminalKey;
use modalkit::keybindings::{BindingMachine, ModalMachine};
use modalkit::prelude::{
    CommandType, CompletionDisplay, CompletionSelection, CompletionType, Count, EditTarget,
    MoveDir1D, MoveType, RepeatType, Specifier,
};
use rooibos_reactive::graph::owner::{StoredValue, on_cleanup, provide_context, use_context};
use rooibos_reactive::graph::traits::{ReadValue, WriteValue};
use rooibos_runtime::InputMode;
use terminput::Event;
use unicode_width::UnicodeWidthStr;
use wasm_compat::cell::UsizeCell;
use wasm_compat::sync::Mutex;

use crate::{keys, parse};

#[cfg(feature = "runtime")]
pub trait CommandFilter<T>
where
    T: CommandCompleter + ApplicationAction,
{
    fn handle_commands(self, handler: CommandHandler<T>) -> Self;
}

#[cfg(feature = "runtime")]
impl<T> CommandFilter<T> for rooibos_runtime::RuntimeSettings
where
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    fn handle_commands(self, mut handler: CommandHandler<T>) -> Self {
        self.event_filter(move |event, input_mode| handler.event_filter(event, input_mode))
    }
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

pub struct AppStore {}

impl ApplicationStore for AppStore {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppId {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppContentId {
    Command(CommandType),
}

impl ApplicationContentId for AppContentId {}

impl ApplicationWindowId for AppId {}

pub trait CommandCompleter {
    fn complete(text: &str, cursor_position: usize) -> Vec<String>;
}

impl CommandCompleter for () {
    fn complete(_text: &str, _cursor_position: usize) -> Vec<String> {
        Vec::new()
    }
}

impl<T> ApplicationInfo for AppInfo<T>
where
    T: ApplicationAction + CommandCompleter,
{
    type Error = String;

    type Action = T;

    type Store = AppStore;

    type WindowId = AppId;

    type ContentId = AppContentId;

    fn content_of_command(cmdtype: CommandType) -> Self::ContentId {
        AppContentId::Command(cmdtype)
    }

    fn complete(
        text: &EditRope,
        cursor: &mut Cursor,
        _content: &Self::ContentId,
        _store: &mut Store<Self>,
    ) -> Vec<String> {
        T::complete(&text.to_string(), cursor.x)
    }
}

pub struct CommandHandler<T>
where
    T: ApplicationAction + CommandCompleter,
{
    manager: KeyManager<TerminalKey, Action<AppInfo<T>>, RepeatType>,
    cmds: CommandMachine<VimCommand<AppInfo<T>>>,
    context: CommandBarContext<T>,
    command_focused: bool,
    action_stack: VecDeque<(Action<AppInfo<T>>, EditContext)>,
}

impl<T> Default for CommandHandler<T>
where
    T: ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

type CommandHandlerActionFn<T> = dyn Fn(CommandHandlerAction<T>, &EditContext) -> Vec<(Action<AppInfo<T>>, EditContext)>
    + Send
    + Sync;

#[derive(Educe)]
#[educe(Clone, Copy)]
pub struct CommandBarContext<T>
where
    T: ApplicationAction + CommandCompleter,
{
    store: StoredValue<Store<AppInfo<T>>>,
    action_handlers: StoredValue<Vec<Box<CommandHandlerActionFn<T>>>>,
    pub(crate) command_handlers: StoredValue<HashMap<usize, CommandHandlerFn<T>>>,
}

impl<T> CommandBarContext<T>
where
    T: ApplicationAction + CommandCompleter + 'static,
{
    pub fn on_command_bar_action<F>(&self, f: F)
    where
        F: Fn(CommandHandlerAction<T>, &EditContext) -> Vec<(Action<AppInfo<T>>, EditContext)>
            + Send
            + Sync
            + 'static,
    {
        let mut handlers = self.action_handlers.write_value();
        handlers.push(Box::new(f))
    }

    pub fn store(&self) -> StoredValue<Store<AppInfo<T>>> {
        self.store
    }

    pub fn dispatch(&self, command: T) {
        let mut handlers = self.command_handlers.write_value();
        for handler in handlers.values_mut() {
            handler.handler.lock_mut()(&command);
        }
    }
}

pub fn use_command_context<T>() -> CommandBarContext<T>
where
    T: ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    use_context::<CommandBarContext<T>>().unwrap()
}

pub fn provide_command_context<T>()
where
    T: ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    if use_context::<CommandBarContext<T>>().is_none() {
        provide_context(CommandBarContext {
            store: StoredValue::new(Store::<AppInfo<T>>::new(AppStore {})),
            action_handlers: StoredValue::new(Vec::new()),
            command_handlers: StoredValue::new(HashMap::new()),
        });
    }
}

pub enum CommandHandlerAction<T>
where
    T: ApplicationAction + CommandCompleter,
{
    CommandBar(CommandBarAction<AppInfo<T>>),
    Editor(EditorAction),
    Prompt(PromptAction),
}

pub trait CommandGenerator<T>
where
    T: ApplicationAction + CommandCompleter,
{
    fn generate_commands(command_handler: &mut CommandHandler<T>);
}

impl<T> CommandHandler<T>
where
    T: ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    pub fn new() -> Self {
        provide_command_context::<T>();
        let mut ism = ModalMachine::<TerminalKey, InputStep<AppInfo<T>>>::empty();
        let colon = parse(":");
        let esc = parse(keys::ESC);
        let enter = parse(keys::ENTER);
        let up = parse(keys::UP);
        let down = parse(keys::DOWN);
        let left = parse(keys::LEFT);
        let right = parse(keys::RIGHT);
        let tab = parse(keys::TAB);
        let shift_tab = parse(keys::combine([keys::SHIFT, keys::TAB]));
        let backspace = parse(keys::BACKSPACE);

        ism.add_mapping(
            VimMode::Normal,
            &colon,
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
            &esc,
            &InputStep::<AppInfo<T>>::new()
                .actions(vec![Action::CommandBar(CommandBarAction::Unfocus)])
                .goto(VimMode::Normal),
        );
        ism.add_mapping(
            VimMode::Command,
            &enter,
            &InputStep::<AppInfo<T>>::new()
                .actions(vec![Action::Prompt(PromptAction::Submit)])
                .goto(VimMode::Normal),
        );
        ism.add_mapping(
            VimMode::Command,
            &up,
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Prompt(PromptAction::Recall(
                MoveDir1D::Previous,
                Count::Contextual,
                true,
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &down,
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Prompt(PromptAction::Recall(
                MoveDir1D::Next,
                Count::Contextual,
                true,
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &left,
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
            &right,
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Editor(EditorAction::Edit(
                Specifier::Contextual,
                EditTarget::Motion(MoveType::Column(MoveDir1D::Next, true), Count::Contextual),
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &backspace,
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Editor(EditorAction::Edit(
                Specifier::Exact(EditAction::Delete),
                EditTarget::Motion(
                    MoveType::Column(MoveDir1D::Previous, true),
                    Count::Contextual,
                ),
            ))]),
        );

        ism.add_mapping(
            VimMode::Command,
            &tab,
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Editor(EditorAction::Complete(
                CompletionType::Auto,
                CompletionSelection::List(MoveDir1D::Next),
                CompletionDisplay::None,
            ))]),
        );
        ism.add_mapping(
            VimMode::Command,
            &shift_tab,
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Editor(EditorAction::Complete(
                CompletionType::Auto,
                CompletionSelection::List(MoveDir1D::Previous),
                CompletionDisplay::None,
            ))]),
        );

        let cmds = VimCommandMachine::<AppInfo<T>>::default();

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

    pub fn event_filter(&mut self, event: Event, mode: InputMode) -> Option<Event> {
        if mode == InputMode::Insert {
            return Some(event);
        }

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
                    let handlers = self.context.action_handlers.read_value();
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
                    let handlers = self.context.action_handlers.read_value();
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
                    let handlers = self.context.action_handlers.read_value();
                    for handler in handlers.iter() {
                        let actions = (handler)(
                            CommandHandlerAction::Prompt(prompt_action.clone()),
                            &context,
                        );
                        new_actions.push(actions);
                    }
                    handled = true;
                }
                Action::Application(app_action) => {
                    let mut handlers = self.context.command_handlers.write_value();
                    for handler in handlers.values_mut() {
                        handler.handler.lock_mut()(&app_action);
                    }
                }
                _ => {}
            }
            for new_action in new_actions {
                self.action_prepend(new_action);
            }
        }

        if handled { None } else { Some(event) }
    }
}

impl<T> CommandHandler<T>
where
    T: ApplicationAction + CommandCompleter + CommandGenerator<T> + Send + Sync + 'static,
{
    pub fn generate_commands(&mut self) {
        T::generate_commands(self);
    }
}

pub fn generate_commands<C>(command_handler: &mut CommandHandler<C>)
where
    C: Parser + ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    let cmd = C::command();
    for sub in cmd.get_subcommands().map(|s| s.get_name()) {
        command_handler.add_command(VimCommand {
            name: sub.to_string(),
            aliases: vec![],
            f: handler,
        });
    }
}

pub fn handler<C>(
    desc: CommandDescription,
    ctx: &mut CommandContext,
) -> CommandResult<VimCommand<AppInfo<C>>>
where
    C: Parser + ApplicationAction + CommandCompleter,
{
    let full_cmd = format!("- {} {}", desc.name(), desc.arg.text);
    let args = shlex::split(&full_cmd).unwrap();
    let action = C::try_parse_from(args).unwrap();
    Ok(CommandStep::Continue(
        Action::Application(action),
        ctx.context.clone(),
    ))
}

pub fn complete<C>(text: &str, cursor_position: usize) -> Vec<String>
where
    C: Parser,
{
    let cursor_position = char_index_to_offset(text, cursor_position);
    let text = &text[..cursor_position];
    let mut cmd = C::command();

    if text.is_empty() {
        return cmd
            .get_subcommands()
            .map(|s| s.get_name().to_string())
            .collect();
    }

    let args: Vec<_> = shlex::split(&format!("- {text}"))
        .unwrap()
        .into_iter()
        .map(OsString::from)
        .collect();

    let pos = if text.ends_with(" \n") {
        args.len()
    } else {
        args.len() - 1
    };
    // Note: trailing spaces aren't currently handled
    // See https://github.com/clap-rs/clap/issues/5587
    let completions =
        clap_complete::engine::complete(&mut cmd, args, pos, None).unwrap_or_default();
    let cursor_word_pos = if let Some(space_pos) = text.rfind(" ") {
        let space_pos = char_index_to_offset(text, space_pos);
        cursor_position - space_pos - 1
    } else {
        cursor_position
    };
    completions
        .into_iter()
        .map(|c| c.get_value().to_string_lossy()[cursor_word_pos..].to_string())
        .collect()
}

fn char_index_to_offset(s: &str, char_index: usize) -> usize {
    s[..s.char_indices().map(|(i, _)| i).nth(char_index).unwrap()].width_cjk()
}

type HandlerFn<T> = dyn FnMut(&T) + Send + Sync;

#[derive(Clone)]
pub struct CommandHandlerFn<T> {
    pub(crate) handler: Arc<Mutex<Box<HandlerFn<T>>>>,
}

static HANDLER_ID: UsizeCell = UsizeCell::new(0);

pub fn on_command<EC, V, T, H>(extract_command: EC, mut handler: H)
where
    EC: Fn(&T) -> Option<V> + Send + Sync + 'static,
    H: FnMut(V) + Send + Sync + 'static,
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    let handler_fn = CommandHandlerFn {
        handler: Arc::new(Mutex::new(Box::new(move |command| {
            if let Some(extracted) = extract_command(command) {
                handler(extracted);
            }
        }))),
    };
    let context = use_command_context::<T>();
    let id = HANDLER_ID.add(1);
    context
        .command_handlers
        .write_value()
        .insert(id, handler_fn);

    on_cleanup(move || {
        context.command_handlers.write_value().remove(&id);
    });
}

#[macro_export]
macro_rules! extract {
    ($v:ident, $pattern:pat $(if $guard:expr)? $(,)?) => {
        |c: &_| if let $pattern $(if $guard)? = c { Some($v.to_owned()) } else { None }
    };
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        |c: &_| if let $pattern $(if $guard)? = c { Some(c.to_owned()) } else { None }
    };

}
