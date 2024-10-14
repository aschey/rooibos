mod command_bar;
mod command_handler;
mod key_handler;

pub use command_bar::*;
pub use command_handler::*;
pub use key_handler::*;
pub use modalkit::actions::Action;
pub use modalkit::editing::application::ApplicationAction;
pub use modalkit::editing::context::EditContext;
use modalkit::env::CommonKeyClass;
pub use modalkit::key::TerminalKey;
pub use modalkit::keybindings::SequenceStatus;
use modalkit::keybindings::{EdgeEvent, EdgeRepeat};
pub use rooibos_keybind_macros::*;

pub fn once(key: &TerminalKey) -> (EdgeRepeat, EdgeEvent<TerminalKey, CommonKeyClass>) {
    (EdgeRepeat::Once, EdgeEvent::Key(*key))
}
