#![doc = include_str!("../../../README.md")]

// fn app() {
//   let handler = handle_command(Command::DoTheThing, do_the_thing);
//
//   let bindings = [
//     bind("C-x", "do the thing", do_the_thing).show(false),
//     bind_cmd("C-x", "do the thing", handler)
//   ];
//   button().on_key_down(bindings)
// }

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
