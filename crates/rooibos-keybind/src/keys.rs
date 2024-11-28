use std::borrow::Cow;
use std::fmt;

use rooibos_reactive::graph::wrappers::read::MaybeSignal;

pub enum Key<'a> {
    Modifier(char),
    Special(Cow<'a, str>),
    Literal(char),
    Class(Cow<'a, str>),
}

impl Key<'_> {
    pub fn decimal(pat: char) -> Self {
        Self::Class(format!("dec{pat}").into())
    }
}

impl fmt::Display for Key<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Modifier(m) => write!(f, "<{m}>"),
            Self::Special(s) => write!(f, "<{s}>"),
            Self::Literal(l) => write!(f, "{l}"),
            Self::Class(c) => write!(f, "{{{c}}}"),
        }
    }
}

pub fn combine<'a, I>(iter: I) -> String
where
    I: IntoIterator<Item = Key<'a>>,
{
    let keys: Vec<_> = iter.into_iter().collect();
    let all_literals = keys
        .iter()
        .all(|k| matches!(k, Key::Literal(_) | Key::Class(_)));
    if all_literals {
        keys.into_iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join("")
    } else {
        let joined = keys
            .into_iter()
            .map(|k| match k {
                Key::Modifier(m) => m.to_string(),
                Key::Special(s) => s.to_string(),
                Key::Literal(l) => l.to_string(),
                Key::Class(c) => c.to_string(),
            })
            .collect::<Vec<_>>()
            .join("-");
        format!("<{joined}>")
    }
}

impl From<Key<'_>> for MaybeSignal<String> {
    fn from(val: Key<'_>) -> Self {
        val.to_string().into()
    }
}

pub const ENTER: Key = Key::Special(Cow::Borrowed("Enter"));
pub const ESC: Key = Key::Special(Cow::Borrowed("Esc"));
pub const UP: Key = Key::Special(Cow::Borrowed("Up"));
pub const DOWN: Key = Key::Special(Cow::Borrowed("Down"));
pub const LEFT: Key = Key::Special(Cow::Borrowed("Left"));
pub const RIGHT: Key = Key::Special(Cow::Borrowed("Right"));
pub const TAB: Key = Key::Special(Cow::Borrowed("Tab"));
pub const BACKSPACE: Key = Key::Special(Cow::Borrowed("BS"));

pub const SHIFT: Key = Key::Modifier('S');
pub const CTRL: Key = Key::Modifier('C');
pub const META: Key = Key::Modifier('M');

pub const DECIMAL: Key = Key::Class(Cow::Borrowed("dec"));
pub const ANY: Key = Key::Class(Cow::Borrowed("any"));
