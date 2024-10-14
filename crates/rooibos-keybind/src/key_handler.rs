use std::collections::HashMap;

use modalkit::actions::{Action, MacroAction};
use modalkit::editing::application::ApplicationAction;
use modalkit::editing::key::KeyManager;
use modalkit::env::vim::VimMode;
use modalkit::env::vim::keybindings::{InputStep, VimMachine};
use modalkit::key::TerminalKey;
use modalkit::keybindings::BindingMachine;
use modalkit::prelude::{Count, RepeatType};
use rooibos_dom::{EventData, EventHandle, KeyHandler};
use terminput::KeyEvent;

use crate::{
    AppInfo, CommandBarContext, CommandCompleter, once, provide_command_context,
    use_command_context,
};

pub struct KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction,
{
    bindings: VimMachine<TerminalKey, AppInfo<T>>,
    mappings: HashMap<String, Box<dyn KeyHandler>>,
}

impl<T> Default for KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction,
{
    pub fn new() -> Self {
        Self {
            bindings: VimMachine::empty(),
            mappings: Default::default(),
        }
    }
    pub fn map_action(&mut self, key: &TerminalKey, action: T) {
        self.bindings.add_mapping(
            VimMode::Normal,
            &[once(key)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Application(action)]),
        );
    }

    pub fn map_handler<H>(&mut self, key: &TerminalKey, handler: H)
    where
        H: KeyHandler + 'static,
    {
        let macro_key = format!("__internal:{key}");
        self.bindings.add_mapping(
            VimMode::Normal,
            &[once(key)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Macro(MacroAction::Run(
                macro_key.clone(),
                Count::Contextual,
            ))]),
        );
        self.mappings.insert(macro_key, Box::new(handler));
    }
}

pub struct KeyInputHandler<T>
where
    T: CommandCompleter + ApplicationAction,
{
    manager: KeyManager<TerminalKey, Action<AppInfo<T>>, RepeatType>,
    command_context: CommandBarContext<T>,
    mappings: HashMap<String, Box<dyn KeyHandler>>,
}

impl<T> KeyInputHandler<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    pub fn new(mapper: KeyMapper<T>) -> Self {
        provide_command_context::<T>();

        Self {
            manager: KeyManager::new(mapper.bindings),
            command_context: use_command_context(),
            mappings: mapper.mappings,
        }
    }

    fn read(&mut self, key_event: KeyEvent, data: EventData, handle: EventHandle) {
        let crossterm_event: Result<crossterm::event::KeyEvent, _> = key_event.try_into();
        let Ok(crossterm_event) = crossterm_event else {
            return;
        };

        self.manager.input_key(crossterm_event.into());

        while let Some((action, context)) = self.manager.pop() {
            match action {
                Action::Application(app_action) => {
                    let mut handlers = self.command_context.command_handlers.lock_mut();
                    for handler in handlers.values_mut() {
                        handler.handler.lock_mut()(&app_action);
                    }
                }
                Action::Macro(macro_action) => {
                    if let MacroAction::Run(name, _) = &macro_action {
                        if let Some(handler) = self.mappings.get_mut(name) {
                            handler.handle(key_event, data, handle.clone());
                            continue;
                        }
                    }
                    self.manager
                        .macro_command(
                            &macro_action,
                            &context,
                            &mut self.command_context.store().write_value(),
                        )
                        .unwrap();
                }
                _ => {}
            }
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

impl<T> KeyHandler for KeyInputHandler<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    fn handle(&mut self, event: KeyEvent, data: EventData, handle: EventHandle) {
        self.read(event, data, handle)
    }
}
