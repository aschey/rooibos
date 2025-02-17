mod command_bar;
mod command_handler;
mod key_handler;
pub mod keys;

use std::fmt;

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
#[cfg(feature = "derive-commands")]
pub use rooibos_keybind_macros::*;

fn parse<S>(input: S) -> Vec<(EdgeRepeat, EdgeEvent<TerminalKey, CommonKeyClass>)>
where
    S: fmt::Display,
{
    modalkit::env::keyparse::parse(&input.to_string())
        .unwrap()
        .1
}
