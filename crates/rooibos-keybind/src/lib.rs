mod command_bar;
mod command_handler;
mod key_handler;

pub use command_bar::*;
pub use command_handler::*;
pub use key_handler::*;
use modalkit::env::CommonKeyClass;
use modalkit::key::TerminalKey;
use modalkit::keybindings::{EdgeEvent, EdgeRepeat};

pub fn once(key: &TerminalKey) -> (EdgeRepeat, EdgeEvent<TerminalKey, CommonKeyClass>) {
    (EdgeRepeat::Once, EdgeEvent::Key(*key))
}
