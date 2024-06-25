use std::error::Error;

use rooibos::components::Button;
use rooibos::dom::{col, derive_signal, row, Constrainable, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::{AppServer, ArcHandle, KeyPair, SshConfig, SshHandler};
use rooibos::tui::text::Text;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let server = AppServer::new(
        SshConfig {
            keys: vec![KeyPair::generate_ed25519().unwrap()],
            ..Default::default()
        },
        SshApp,
    );

    server.run(("0.0.0.0", 2222)).await?;
    Ok(())
}

struct SshApp;

impl SshHandler for SshApp {
    #[allow(refining_impl_trait)]
    async fn run_terminal(
        &self,
        _client_id: u32,
        handle: ArcHandle,
        event_rx: tokio::sync::mpsc::Receiver<rooibos::dom::Event>,
        _client_addr: Option<std::net::SocketAddr>,
    ) {
        let runtime = Runtime::initialize(
            RuntimeSettings::default(),
            SshBackend::new(
                CrosstermBackend::new(TerminalSettings::new(move || handle.clone())),
                event_rx,
            ),
            app,
        );
        runtime.run().await.unwrap();
    }
}

fn app() -> impl Render {
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    row![
        Button::new()
            .length(20)
            .on_click(move || set_count.update(|c| *c += 1))
            .render(derive_signal!(Text::from(format!("count {}", count.get()))))
    ]
    .length(3)
}
