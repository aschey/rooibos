#![doc = include_str!("../../../README.md")]

pub mod reactive {
    pub use rooibos_reactive::*;
}

pub mod runtime {
    pub use rooibos_runtime::*;
}

pub mod terminal {
    pub use rooibos_terminal::*;
}

pub mod components {
    pub use rooibos_components::*;
}

pub mod theme {
    pub use rooibos_theme::*;
}

#[cfg(feature = "config")]
pub mod config {
    pub use rooibos_config::*;
}

#[cfg(feature = "router")]
pub mod router {
    pub use rooibos_router::*;
}

#[cfg(feature = "keybind")]
pub mod keybind {
    pub use rooibos_keybind::*;
}

#[cfg(feature = "tester")]
pub mod tester {
    pub use rooibos_tester::*;
}

#[cfg(feature = "ssh")]
pub mod ssh {
    pub use rooibos_ssh::*;
}

#[cfg(feature = "web")]
pub mod web {
    #[cfg(target_arch = "wasm32")]
    pub use rooibos_web::*;
}

pub use ratatui as tui;
pub use rooibos_reactive_macros::*;
