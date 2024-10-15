use std::collections::HashMap;
use std::sync::Arc;

use modalkit::actions::{Action, MacroAction};
use modalkit::editing::application::ApplicationAction;
use modalkit::editing::key::KeyManager;
use modalkit::env::CommonKeyClass;
use modalkit::env::vim::VimMode;
use modalkit::env::vim::keybindings::{InputStep, VimMachine};
use modalkit::key::TerminalKey;
use modalkit::keybindings::{BindingMachine, EdgePathPart};
use modalkit::prelude::{Count, RepeatType};
use rooibos_dom::{KeyEventProps, KeyHandler};
use rooibos_reactive::graph::computed::Memo;
use rooibos_reactive::graph::effect::Effect;
use rooibos_reactive::graph::signal::{WriteSignal, signal};
use rooibos_reactive::graph::traits::{Get, Update, With, WriteValue};
use rooibos_reactive::graph::wrappers::read::{MaybeSignal, Signal};
use wasm_compat::sync::Mutex;

use crate::{
    AppInfo, CommandBarContext, CommandCompleter, provide_command_context, use_command_context,
};

#[derive(Clone)]
struct KeyMapHolder<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync,
{
    bindings: Arc<Mutex<KeyManager<TerminalKey, Action<AppInfo<T>>, RepeatType>>>,
    mappings: HashMap<String, Arc<Mutex<Box<dyn KeyHandler + Send + Sync>>>>,
}

impl<T> PartialEq for KeyMapHolder<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync,
{
    // this is required to use with Memo, but KeyManager doesn't implement Eq so we don't have a
    // good way of implementing this
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T> Eq for KeyMapHolder<T> where T: CommandCompleter + ApplicationAction + Send + Sync {}

pub struct KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    bindings: Signal<KeyMapHolder<T>>,
    set_key_maps: WriteSignal<HashMap<String, InternalKeyMap<T>>>,
}

impl<T> Default for KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, KM> From<KM> for KeyMapper<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync,
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
    T: CommandCompleter + ApplicationAction + Send + Sync,
{
    pub fn new() -> Self {
        let (key_maps, set_key_maps) = signal(HashMap::<String, InternalKeyMap<T>>::new());
        let bindings = Memo::new(move |_| {
            let mut ism = VimMachine::empty();
            let mut mappings = HashMap::default();
            key_maps.with(|k| {
                for key_map in k.values() {
                    match key_map {
                        InternalKeyMap::Action(key, action) => {
                            ism.add_mapping(
                                VimMode::Normal,
                                key,
                                &InputStep::<AppInfo<T>>::new()
                                    .actions(vec![Action::Application(action.clone())]),
                            );
                        }
                        InternalKeyMap::Handler(key, handler) => {
                            let macro_key = format!("__internal:{key:?}");
                            ism.add_mapping(
                                VimMode::Normal,
                                key,
                                &InputStep::<AppInfo<T>>::new().actions(vec![Action::Macro(
                                    MacroAction::Run(macro_key.clone(), Count::Contextual),
                                )]),
                            );
                            mappings.insert(macro_key, handler.clone());
                        }
                    }
                }
            });
            KeyMapHolder {
                bindings: Arc::new(Mutex::new(KeyManager::new(ism))),
                mappings,
            }
        });
        Self {
            bindings: bindings.into(),
            set_key_maps,
        }
    }

    fn map(&mut self, map: KeyMap<T>) {
        match map {
            KeyMap::Action(key, action) => {
                self.map_action(key, action);
            }
            KeyMap::Handler(key, handler) => {
                self.map_handler_inner(key, handler);
            }
        }
    }

    pub fn map_action<S>(&mut self, key: S, action: T)
    where
        S: Into<MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>>,
    {
        let key = key.into();
        let mut action = Some(action);
        let set_key_maps = self.set_key_maps;
        Effect::new(move |prev| {
            let new_key = key.get();
            let new_key_str = format!("{new_key:?}");
            if let Some(prev) = prev {
                set_key_maps.update(|m| {
                    let map = m.remove(&prev).unwrap();
                    let InternalKeyMap::Action(_, action) = map else {
                        unreachable!();
                    };
                    m.insert(new_key_str.clone(), InternalKeyMap::Action(new_key, action));
                });
            } else {
                set_key_maps.update(|m| {
                    m.insert(
                        new_key_str.clone(),
                        InternalKeyMap::Action(new_key, action.take().unwrap()),
                    );
                });
            }
            new_key_str
        });
    }

    pub fn map_handler<S, H>(&mut self, key: S, handler: H)
    where
        S: Into<MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>>,
        H: KeyHandler + Send + Sync + 'static,
    {
        let handler = Arc::new(Mutex::new(
            Box::new(handler) as Box<dyn KeyHandler + Send + Sync>
        ));
        self.map_handler_inner(key.into(), handler);
    }

    fn map_handler_inner(
        &mut self,
        key: MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>,
        handler: Arc<Mutex<Box<dyn KeyHandler + Send + Sync>>>,
    ) {
        let set_key_maps = self.set_key_maps;
        let mut handler = Some(handler);
        Effect::new(move |prev| {
            let new_key = key.get();
            let new_key_str = format!("{new_key:?}");

            if let Some(prev) = prev {
                set_key_maps.update(|m| {
                    let map = m.remove(&prev).unwrap();
                    let InternalKeyMap::Handler(_, handler) = map else {
                        unreachable!();
                    };
                    m.insert(
                        new_key_str.clone(),
                        InternalKeyMap::Handler(new_key, handler),
                    );
                });
            } else {
                set_key_maps.update(|m| {
                    m.insert(
                        new_key_str.clone(),
                        InternalKeyMap::Handler(new_key, handler.take().unwrap()),
                    );
                });
            }
            new_key_str
        });
    }
}

pub enum KeyMap<T> {
    Action(
        MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>,
        T,
    ),
    Handler(
        MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>,
        Arc<Mutex<Box<dyn KeyHandler + Send + Sync>>>,
    ),
}

enum InternalKeyMap<T> {
    Action(Vec<EdgePathPart<TerminalKey, CommonKeyClass>>, T),
    Handler(
        Vec<EdgePathPart<TerminalKey, CommonKeyClass>>,
        Arc<Mutex<Box<dyn KeyHandler + Send + Sync>>>,
    ),
}

pub fn map_action<S, T>(key: S, action: T) -> KeyMap<T>
where
    S: Into<MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>>,
{
    KeyMap::Action(key.into(), action)
}

pub fn map_handler<S, T, H>(key: S, handler: H) -> KeyMap<T>
where
    S: Into<MaybeSignal<Vec<EdgePathPart<TerminalKey, CommonKeyClass>>>>,
    H: KeyHandler + Send + Sync + 'static,
{
    KeyMap::Handler(key.into(), Arc::new(Mutex::new(Box::new(handler))))
}

pub struct KeyInputHandler<T>
where
    T: CommandCompleter + ApplicationAction + Send + Sync + 'static,
{
    manager: Signal<KeyMapHolder<T>>,
    command_context: CommandBarContext<T>,
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
            manager: mapper.bindings,
            command_context: use_command_context(),
        }
    }

    fn read(&mut self, props: KeyEventProps) {
        let crossterm_event: Result<crossterm::event::KeyEvent, _> = props.event.try_into();
        let Ok(crossterm_event) = crossterm_event else {
            return;
        };

        let mut manager = self.manager.get();
        let mappings = &mut manager.mappings;
        let mut manager = manager.bindings.lock_mut();
        manager.input_key(crossterm_event.into());

        while let Some((action, context)) = manager.pop() {
            match action {
                Action::Application(app_action) => {
                    let mut handlers = self.command_context.command_handlers.lock_mut();
                    for handler in handlers.values_mut() {
                        handler.handler.lock_mut()(&app_action);
                    }
                }
                Action::Macro(macro_action) => {
                    if let MacroAction::Run(name, _) = &macro_action {
                        if let Some(handler) = mappings.get_mut(name) {
                            handler.lock_mut().handle(props.clone());
                            continue;
                        }
                    }
                    manager
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
