pub mod backend;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::Arc;

use async_signal::{Signal, Signals};
use background_service::Manager;
use futures::{Future, StreamExt};
use futures_cancel::FutureExt;
use ratatui::backend::WindowSize;
use ratatui::layout::Size;
use rooibos_dom::Event;
use rooibos_reactive::graph::owner::Owner;
use rooibos_reactive::init_executor;
use rooibos_runtime::{
    CancellationToken, ServiceContext, restore_terminal, set_external_signal_source, with_runtime,
};
pub use russh::keys;
use russh::keys::PublicKey;
pub use russh::server::Config as SshConfig;
use russh::server::{Auth, Handle, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};
use tap::TapFallible;
use tokio::net::ToSocketAddrs;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::task::LocalSet;
use tracing::warn;

pub struct TerminalHandle {
    handle: Handle,
    // The sink collects the data which is finally flushed to the handle.
    sink: Vec<u8>,
    channel_id: ChannelId,
}

impl TerminalHandle {
    pub(crate) fn new(handle: Handle, channel_id: ChannelId) -> Self {
        Self {
            handle,
            channel_id,
            sink: Vec::new(),
        }
    }
}

// The crossterm backend writes to the terminal handle.
impl std::io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let handle = self.handle.clone();
        let channel_id = self.channel_id;
        let data = self.sink.clone().into();
        futures::executor::block_on(async move {
            let result = handle.data(channel_id, data).await;
            if result.is_err() {
                warn!("Failed to send data: {result:?}");
            }
        });

        self.sink.clear();
        Ok(())
    }
}

#[derive(Clone)]
pub struct ArcHandle(Arc<std::sync::RwLock<TerminalHandle>>);

impl std::io::Write for ArcHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.write().unwrap().flush()
    }
}

pub struct AppServer<T>
where
    T: SshHandler,
{
    clients: Arc<RwLock<HashMap<u32, SshEventSender>>>,
    handler: Arc<T>,
    config: Arc<SshConfig>,
    service_manager: Option<Manager>,
    service_context: ServiceContext,
    current_client_id: u32,
}

impl<T: SshHandler> AppServer<T> {
    pub fn new(config: SshConfig, handler: T) -> Self {
        init_executor();
        let service_manager = Manager::new(
            CancellationToken::new(),
            background_service::Settings::default(),
        );
        let service_context = service_manager.get_context();
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            handler: Arc::new(handler),
            config: Arc::new(config),
            service_manager: Some(service_manager),
            service_context,
            current_client_id: 0,
        }
    }

    pub async fn run<A: ToSocketAddrs + Send + 'static>(mut self, address: A) -> io::Result<()> {
        #[cfg(unix)]
        // SIGSTP cannot be handled https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html
        let mut signals = Signals::new([Signal::Term, Signal::Quit, Signal::Int]).unwrap();
        #[cfg(windows)]
        let mut signals = Signals::new([Signal::Int]).unwrap();

        let (signal_tx, mut signal_rx) = broadcast::channel(32);
        set_external_signal_source(signal_tx.clone()).expect("signal handler already set");

        let service_manager = self.service_manager.take().unwrap();
        let service_context = self.service_context.clone();
        service_context.spawn((
            "ssh_signal_handler",
            move |context: ServiceContext| async move {
                while let Ok(Some(Ok(signal))) =
                    signals.next().cancel_with(context.cancelled()).await
                {
                    signal_tx.send(signal).unwrap();
                }
                Ok(())
            },
        ));

        service_context.spawn(("ssh_server", move |context: ServiceContext| async move {
            let _ = self
                .run_on_address(self.config.clone(), address)
                .cancel_with(context.cancelled())
                .await;
            Ok(())
        }));

        signal_rx.recv().await.unwrap();
        service_context.cancel_all();
        service_manager.join_on_cancel().await.unwrap();

        Ok(())
    }
}

pub trait SshHandler: Send + Sync + 'static {
    fn run_terminal(
        &self,
        client_id: u32,
        handle: ArcHandle,
        events: SshEventReceiver,
        client_addr: Option<std::net::SocketAddr>,
    ) -> impl Future + Send;
}

pub struct SshEventSender {
    events: mpsc::Sender<Event>,
    window_size: Arc<std::sync::RwLock<WindowSize>>,
}

pub struct SshEventReceiver {
    events: mpsc::Receiver<Event>,
    window_size: Arc<std::sync::RwLock<WindowSize>>,
}

pub struct AppHandler<T>
where
    T: SshHandler,
{
    client_id: u32,
    clients: Arc<RwLock<HashMap<u32, SshEventSender>>>,
    handler: Arc<T>,
    socket_addr: Option<std::net::SocketAddr>,
    service_context: ServiceContext,
}

impl<T> AppHandler<T>
where
    T: SshHandler,
{
    async fn handle_terminal_size(&self, window_size: WindowSize) {
        let clients = self.clients.read().await;
        if let Some(sender) = clients.get(&self.client_id) {
            *sender.window_size.write().unwrap() = window_size;
            let _ = sender
                .events
                .send(Event::Resize {
                    rows: window_size.columns_rows.height as u32,
                    cols: window_size.columns_rows.width as u32,
                })
                .await
                .tap_err(|e| warn!("error sending data: {e:?}"));
        }
    }
}

impl<T> Server for AppServer<T>
where
    T: SshHandler,
{
    type Handler = AppHandler<T>;
    fn new_client(&mut self, socket_addr: Option<std::net::SocketAddr>) -> AppHandler<T> {
        self.current_client_id += 1;
        AppHandler {
            client_id: self.current_client_id,
            handler: self.handler.clone(),
            clients: self.clients.clone(),
            socket_addr,
            service_context: self.service_context.clone(),
        }
    }
}

impl<T: SshHandler> Handler for AppHandler<T> {
    type Error = Box<dyn Error + Send + Sync>;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let (event_tx, event_rx) = mpsc::channel(32);

        let clients = self.clients.clone();
        let client_id = self.client_id;
        let window_size = Arc::new(std::sync::RwLock::new(WindowSize {
            columns_rows: Size::default(),
            pixels: Size::default(),
        }));
        clients.write().await.insert(
            client_id,
            SshEventSender {
                events: event_tx,
                window_size: window_size.clone(),
            },
        );

        let handler = self.handler.clone();
        let client_id = self.client_id;
        let channel_id = channel.id();
        let terminal_handle = TerminalHandle::new(session.handle(), channel_id);
        let handle = session.handle();
        let socket_addr = self.socket_addr;
        self.service_context.spawn_blocking(("tui", move |_| {
            let owner = Owner::new();
            owner.with(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();

                rt.block_on(async move {
                    let local_set = LocalSet::new();
                    local_set
                        .run_until(async move {
                            with_runtime(client_id, async move {
                                handler
                                    .run_terminal(
                                        client_id,
                                        ArcHandle(Arc::new(std::sync::RwLock::new(
                                            terminal_handle,
                                        ))),
                                        SshEventReceiver {
                                            events: event_rx,
                                            window_size,
                                        },
                                        socket_addr,
                                    )
                                    .await;
                                restore_terminal().unwrap();
                            })
                            .await
                        })
                        .await;
                    clients.write().await.remove(&client_id);
                    handle.close(channel_id).await.unwrap();
                });
            });
            Ok(())
        }));

        Ok(true)
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.eof(channel).unwrap();
        session
            .disconnect(russh::Disconnect::ByApplication, "Quit", "")
            .unwrap();
        session.close(channel).unwrap();
        Ok(())
    }

    async fn auth_publickey(&mut self, _: &str, _: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(event) = Event::parse_from(data)? {
            let clients = self.clients.read().await;
            if let Some(client_tx) = clients.get(&self.client_id) {
                let _ = client_tx
                    .events
                    .send(event)
                    .await
                    .tap_err(|e| warn!("error sending data: {e:?}"));
            }
        }

        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.handle_terminal_size(WindowSize {
            columns_rows: Size {
                width: col_width as u16,
                height: row_height as u16,
            },
            pixels: Size {
                width: pix_width as u16,
                height: pix_height as u16,
            },
        })
        .await;
        session.channel_success(channel)?;
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.handle_terminal_size(WindowSize {
            columns_rows: Size {
                width: col_width as u16,
                height: row_height as u16,
            },
            pixels: Size {
                width: pix_width as u16,
                height: pix_height as u16,
            },
        })
        .await;
        session.channel_success(channel)?;
        Ok(())
    }
}
