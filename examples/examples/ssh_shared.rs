use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::derive_signal;
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, span};
use rooibos::reactive::graph::signal::ReadSignal;
use rooibos::reactive::graph::traits::{FromStream, Get};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::{AppServer, ArcHandle, KeyPair, SshConfig, SshHandler};
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::WatchStream;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[tokio::main]
async fn main() -> Result {
    let (count_tx, _) = watch::channel(0);
    let server = AppServer::new(
        SshConfig {
            keys: vec![KeyPair::generate_ed25519()],
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
        handle: ArcHandle,
        event_rx: mpsc::Receiver<rooibos::reactive::Event>,
        _client_addr: Option<std::net::SocketAddr>,
    ) {
        let count_tx = self.count_tx.clone();
        Runtime::initialize(SshBackend::new(handle, event_rx))
            .run(|| app(count_tx))
            .await
            .unwrap();
    }
}

fn app(count_tx: watch::Sender<i32>) -> impl Render {
    let count_rx = count_tx.subscribe();
    let count = ReadSignal::from_stream(WatchStream::new(count_rx));
    Button::new()
        .width(chars(20.))
        .height(chars(3.))
        .on_click(move || {
            count_tx.send(count.get().unwrap_or_default() + 1).unwrap();
        })
        .render(derive_signal!(
            line!("count ", span!(count.get().unwrap_or_default())).into()
        ))
}
