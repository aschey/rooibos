use std::process::ExitCode;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rooibos::components::Button;
use rooibos::reactive::derive_signal;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line};
use rooibos::reactive::graph::signal::ReadSignal;
use rooibos::reactive::graph::traits::{FromStream, Get};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::keys::PrivateKey;
use rooibos::ssh::keys::ssh_key::private::{Ed25519Keypair, KeypairData};
use rooibos::ssh::{AppServer, SshConfig, SshHandler, SshParams};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[tokio::main]
async fn main() -> Result {
    let (count_tx, _) = watch::channel(0);
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
        SshApp { count_tx },
    );

    server.run(("0.0.0.0", 2222)).await?;
    Ok(ExitCode::SUCCESS)
}

struct SshApp {
    count_tx: watch::Sender<i32>,
}

impl SshHandler for SshApp {
    #[allow(refining_impl_trait)]
    async fn run_terminal(
        &self,
        _client_id: u32,
        params: SshParams,
        _client_addr: Option<std::net::SocketAddr>,
    ) {
        let count_tx = self.count_tx.clone();
        Runtime::initialize(SshBackend::new(params).await.unwrap())
            .run(|| app(count_tx))
            .await
            .unwrap();
    }
}

fn app(count_tx: watch::Sender<i32>) -> impl Render {
    let count_rx = count_tx.subscribe();
    let count = ReadSignal::from_stream(WatchStream::new(count_rx));
    Button::new()
        .width(20)
        .height(3)
        .on_click(move || {
            count_tx.send(count.get().unwrap_or_default() + 1).unwrap();
        })
        .render(derive_signal!(
            line!("count ", count.get().unwrap_or_default()).into()
        ))
}
