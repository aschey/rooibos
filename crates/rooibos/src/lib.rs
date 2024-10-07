#![doc = include_str!("../../../README.md")]

pub mod reactive {
    pub use rooibos_reactive::*;
}

pub mod dom {
    pub use rooibos_dom::*;
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

#[cfg(feature = "config")]
pub mod config {
    pub use rooibos_config::*;
}

#[cfg(feature = "router")]
pub mod router {
    pub use rooibos_router::*;
}

pub mod tester {
    pub use rooibos_tester::*;
}

#[cfg(feature = "ssh")]
pub mod ssh {
    pub use rooibos_ssh::*;
}

#[cfg(feature = "xterm-js")]
pub mod xterm_js {
    #[cfg(target_arch = "wasm32")]
    pub use rooibos_xterm_js::*;
}
pub use ratatui as tui;
pub use rooibos_reactive_macros::*;
