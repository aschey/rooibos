use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, derive_signal};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::{AppServer, ArcHandle, KeyPair, SshConfig, SshHandler};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[tokio::main]
async fn main() -> Result {
    let server = AppServer::new(
        SshConfig {
            keys: vec![KeyPair::generate_ed25519()],
            ..Default::default()
        },
        SshApp,
    );

    server.run(("0.0.0.0", 2222)).await?;
    Ok(ExitCode::SUCCESS)
}

struct SshApp;

impl SshHandler for SshApp {
    #[allow(refining_impl_trait)]
    async fn run_terminal(
        &self,
        _client_id: u32,
        handle: ArcHandle,
        event_rx: tokio::sync::mpsc::Receiver<rooibos::reactive::Event>,
        _client_addr: Option<std::net::SocketAddr>,
    ) {
        Runtime::initialize(SshBackend::new(handle, event_rx))
            .run(app)
            .await
            .unwrap();
    }
}

fn app() -> impl Render {
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    Button::new()
        .width(chars(20.))
        .height(chars(3.))
        .on_click(move || set_count.update(|c| *c += 1))
        .render(derive_signal!(line!("count ", span!(count.get())).into()))
}
