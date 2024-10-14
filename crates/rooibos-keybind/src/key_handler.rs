use std::collections::HashMap;

use modalkit::actions::{Action, MacroAction};
use modalkit::editing::application::ApplicationAction;
use modalkit::editing::key::KeyManager;
use modalkit::env::vim::VimMode;
use modalkit::env::vim::keybindings::{InputStep, VimMachine};
use modalkit::key::TerminalKey;
use modalkit::keybindings::BindingMachine;
use modalkit::prelude::{Count, RepeatType};
use rooibos_dom::{KeyEventProps, KeyHandler};

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

impl<T, KM> From<KM> for KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction,
    KM: IntoIterator<Item = KeyMap<T>>,
{
    fn from(value: KM) -> Self {
        let mut mapper = KeyMapper::new();
        for val in value.into_iter() {
            mapper.map(val);
        }
        mapper
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

    fn map(&mut self, map: KeyMap<T>) {
        match map {
            KeyMap::Action(key, action) => {
                self.map_action(&key, action);
            }
            KeyMap::Handler(key, handler) => {
                self.map_handler_inner(&key, handler);
            }
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
        self.map_handler_inner(key, Box::new(handler))
    }

    fn map_handler_inner(&mut self, key: &TerminalKey, handler: Box<dyn KeyHandler>) {
        let macro_key = format!("__internal:{key}");
        self.bindings.add_mapping(
            VimMode::Normal,
            &[once(key)],
            &InputStep::<AppInfo<T>>::new().actions(vec![Action::Macro(MacroAction::Run(
                macro_key.clone(),
                Count::Contextual,
            ))]),
        );
        self.mappings.insert(macro_key, handler);
    }
}

pub enum KeyMap<T> {
    Action(TerminalKey, T),
    Handler(TerminalKey, Box<dyn KeyHandler>),
}

pub fn map_action<T>(key: TerminalKey, action: T) -> KeyMap<T> {
    KeyMap::Action(key, action)
}

pub fn map_handler<T, H>(key: TerminalKey, handler: H) -> KeyMap<T>
where
    H: KeyHandler + 'static,
{
    KeyMap::Handler(key, Box::new(handler))
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
    pub fn new<K>(mapper: K) -> Self
    where
        K: Into<KeyMapper<T>>,
    {
        let mapper = mapper.into();
        provide_command_context::<T>();

        Self {
            manager: KeyManager::new(mapper.bindings),
            command_context: use_command_context(),
            mappings: mapper.mappings,
        }
    }

    fn read(&mut self, props: KeyEventProps) {
        let crossterm_event: Result<crossterm::event::KeyEvent, _> = props.event.try_into();
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
                            handler.handle(props.clone());
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
        }
    }
}

impl<T> KeyHandler for KeyInputHandler<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    fn handle(&mut self, props: KeyEventProps) {
        self.read(props)
    }
}
