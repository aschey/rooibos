use rooibos::components::Button;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::layout::chars;
use rooibos::reactive::{Render, UpdateLayoutProps, col, derive_signal, line, mount, span};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::ssh::backend::SshBackend;
use rooibos::ssh::{AppServer, ArcHandle, KeyPair, SshConfig, SshHandler};
type Result<T> = std::result::Result<T, RuntimeError>;

#[tokio::main]
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
        mount(app);
        let runtime = Runtime::initialize(SshBackend::new(handle, event_rx));
        runtime.run().await.unwrap();
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
