use std::error::Error;

use rooibos::components::Button;
use rooibos::dom::{col, derive_signal, row, Constrainable, Render};
use rooibos::reactive::signal::ReadSignal;
use rooibos::reactive::traits::Get;
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::{AppServer, ArcHandle, KeyPair, SshConfig, SshHandler};
use rooibos::tui::text::Text;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let (count_tx, _) = watch::channel(0);
    let server = AppServer::new(
        SshConfig {
            keys: vec![KeyPair::generate_ed25519().unwrap()],
            ..Default::default()
        },
        SshApp { count_tx },
    );

    server.run(("0.0.0.0", 2222)).await?;
    Ok(())
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
        event_rx: tokio::sync::mpsc::Receiver<rooibos::dom::Event>,
        _client_addr: Option<std::net::SocketAddr>,
    ) {
        let count_tx = self.count_tx.clone();
        let runtime = Runtime::initialize(
            RuntimeSettings::default(),
            SshBackend::new(
                CrosstermBackend::new(TerminalSettings::new(move || handle.clone())),
                event_rx,
            ),
            move || app(count_tx),
        );
        runtime.run().await.unwrap();
    }
}

fn app(count_tx: watch::Sender<i32>) -> impl Render {
    let count_rx = count_tx.subscribe();
    let count = ReadSignal::from_stream(WatchStream::new(count_rx));
    let count = ReadSignal::from(count);
    col![
        row![
            Button::new()
                .length(20)
                .on_click(move || {
                    count_tx.send(count.get().unwrap_or_default() + 1).unwrap();
                })
                .render(derive_signal!(Text::from(format!(
                    "count {}",
                    count.get().unwrap_or_default()
                ))))
        ]
        .length(3)
    ]
}
