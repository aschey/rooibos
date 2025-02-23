use std::process::ExitCode;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rooibos::components::Button;
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, derive_signal};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::keys::PrivateKey;
use rooibos::ssh::keys::ssh_key::private::{Ed25519Keypair, KeypairData};
use rooibos::ssh::{AppServer, ArcHandle, SshConfig, SshHandler};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[tokio::main]
async fn main() -> Result {
    let server = AppServer::new(
        SshConfig {
            keys: vec![
                PrivateKey::new(
                    KeypairData::Ed25519(Ed25519Keypair::random(&mut StdRng::seed_from_u64(42))),
                    "test key",
                )
                .unwrap(),
            ],
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
        .width(chars(20))
        .height(chars(3))
        .on_click(move || set_count.update(|c| *c += 1))
        .render(derive_signal!(line!("count ", span!(count.get())).into()))
}
