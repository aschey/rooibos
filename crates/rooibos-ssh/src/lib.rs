pub mod backend;

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_signal::{Signal, Signals};
use background_service::Manager;
use futures::{Future, StreamExt};
use ratatui::backend::WindowSize;
use ratatui::layout::Size;
use rooibos_dom::Event;
use rooibos_reactive::graph::owner::Owner;
use rooibos_reactive::init_executor;
use rooibos_runtime::{
    CancellationToken, ServiceContext, restore_terminal, set_external_signal_source,
    with_runtime_async,
};
pub use russh::keys;
use russh::keys::PublicKey;
pub use russh::server::Config as SshConfig;
use russh::server::{Auth, Handle, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};
use tap::TapFallible;
use termina::Parser;
use terminput_termina::to_terminput;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::task::LocalSet;
use tokio_util::future::FutureExt;
use tracing::{error, warn};

pub struct TerminalHandle {
    handle: Handle,
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
    server_shutdown: CancellationToken,
    current_client_id: u32,
    shutdown_requested: Arc<AtomicBool>,
}

impl<T: SshHandler> AppServer<T> {
    pub fn new(config: SshConfig, handler: T) -> Self {
        init_executor();
        let service_manager = Manager::new(
            CancellationToken::new(),
            background_service::Settings::default().task_wait_duration(Duration::from_secs(20)),
        );
        let service_context = service_manager.get_context();
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            handler: Arc::new(handler),
            config: Arc::new(config),
            service_manager: Some(service_manager),
            service_context,
            server_shutdown: CancellationToken::new(),
            current_client_id: 0,
            shutdown_requested: AtomicBool::new(false).into(),
        }
    }

    pub async fn run<A: ToSocketAddrs + Send + 'static>(mut self, address: A) -> io::Result<()> {
        #[cfg(unix)]
        // SIGSTP cannot be handled https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html
        let mut signals = Signals::new([Signal::Term, Signal::Quit, Signal::Int]).unwrap();
        #[cfg(windows)]
        let mut signals = Signals::new([Signal::Int]).unwrap();

        let (signal_tx, _) = broadcast::channel(32);
        set_external_signal_source(signal_tx.clone()).expect("signal handler already set");

        let service_manager = self.service_manager.take().unwrap();
        let service_context = self.service_context.clone();
        let shutdown_requested = self.shutdown_requested.clone();
        let clients = self.clients.clone();
        let server_shutdown = self.server_shutdown.clone();
        service_context.spawn((
            "ssh_signal_handler",
            move |context: ServiceContext| async move {
                if let Some(Ok(signal)) = signals
                    .next()
                    .with_cancellation_token(context.cancellation_token())
                    .await
                    .flatten()
                {
                    shutdown_requested.store(true, Ordering::Release);
                    if clients.read().await.is_empty() {
                        // If no clients are connected, we can shut down immediately
                        server_shutdown.cancel();
                    }
                    let _ = signal_tx.send(signal);
                    tokio::spawn(async move {
                        // fallback: If a client exited unexpectedly,
                        // we may not be able to wait for every connection to drain cleanly
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        server_shutdown.cancel();
                    });
                }
                Ok(())
            },
        ));
        let server_shutdown = self.server_shutdown.clone();
        service_context.spawn(("ssh_server", move |context: ServiceContext| async move {
            let socket = TcpListener::bind(address).await?;
            let server = self.run_on_socket(self.config.clone(), &socket);
            let server_handle = server.handle();

            context.spawn(("ssh_shutdown", move |_: ServiceContext| async move {
                server_shutdown.cancelled().await;
                server_handle.shutdown("server shutdown".into());
                Ok(())
            }));

            server.await?;
            context.cancel_all();
            Ok(())
        }));

        service_manager.join_on_cancel().await.unwrap();

        Ok(())
    }
}

pub struct SshParams {
    pub handle: ArcHandle,
    pub events: SshEventReceiver,
    pub term: String,
}

pub trait SshHandler: Send + Sync + 'static {
    fn run_terminal(
        &self,
        client_id: u32,
        params: SshParams,
        client_addr: Option<std::net::SocketAddr>,
    ) -> impl Future + Send;
}

pub struct SshEventSender {
    events: mpsc::Sender<Event>,
    query_events: mpsc::Sender<termina::Event>,
    window_size: Arc<std::sync::RwLock<WindowSize>>,
}

pub struct SshEventReceiver {
    events: mpsc::Receiver<Event>,
    query_events: mpsc::Receiver<termina::Event>,
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
    shutdown_requested: Arc<AtomicBool>,
    server_shutdown: CancellationToken,
    service_context: ServiceContext,
    parser: Parser,
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
            shutdown_requested: self.shutdown_requested.clone(),
            socket_addr,
            server_shutdown: self.server_shutdown.clone(),
            service_context: self.service_context.clone(),
            parser: Parser::default(),
        }
    }

    fn handle_session_error(&mut self, error: <Self::Handler as Handler>::Error) {
        error!("session error: {error:?}");
    }
}

impl<T: SshHandler> Handler for AppHandler<T> {
    type Error = Box<dyn Error + Send + Sync>;

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
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
        let shutdown_requested = self.shutdown_requested.load(Ordering::Acquire);
        let can_shutdown = shutdown_requested && self.clients.read().await.is_empty();
        if can_shutdown {
            session.flush().unwrap();
            session.flush_pending(channel).unwrap();
            self.server_shutdown.cancel();
        }
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
        self.parser.parse(data, false);
        while let Some(event) = self.parser.pop() {
            let clients = self.clients.read().await;
            if let Some(client_tx) = clients.get(&self.client_id) {
                match event {
                    termina::Event::Csi(_) | termina::Event::Dcs(_) => {
                        let _ = client_tx
                            .query_events
                            .send(event)
                            .await
                            .tap_err(|e| warn!("error sending data: {e:?}"));
                    }
                    _ => {
                        if let Ok(event) = to_terminput(event) {
                            let _ = client_tx
                                .events
                                .send(event)
                                .await
                                .tap_err(|e| warn!("error sending data: {e:?}"));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        term: &str,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let (event_tx, event_rx) = mpsc::channel(1024);
        let (query_event_tx, query_event_rx) = mpsc::channel(1024);

        let clients = self.clients.clone();
        let client_id = self.client_id;
        let window_size = Arc::new(std::sync::RwLock::new(WindowSize {
            columns_rows: Size {
                width: col_width as u16,
                height: row_height as u16,
            },
            pixels: Size {
                width: pix_width as u16,
                height: pix_height as u16,
            },
        }));
        clients.write().await.insert(
            client_id,
            SshEventSender {
                events: event_tx,
                query_events: query_event_tx,
                window_size: window_size.clone(),
            },
        );

        let handler = self.handler.clone();

        let terminal_handle = TerminalHandle::new(session.handle(), channel);
        let handle = session.handle();
        let socket_addr = self.socket_addr;

        let term = term.to_string();
        self.service_context
            .spawn_thread(("tui", move |_: ServiceContext| {
                let owner = Owner::new();
                owner.with(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async move {
                        let local_set = LocalSet::new();
                        local_set
                            .run_until(async move {
                                with_runtime_async(client_id, async move {
                                    let params = SshParams {
                                        handle: ArcHandle(Arc::new(std::sync::RwLock::new(
                                            terminal_handle,
                                        ))),
                                        events: SshEventReceiver {
                                            events: event_rx,
                                            query_events: query_event_rx,
                                            window_size,
                                        },
                                        term,
                                    };
                                    handler.run_terminal(client_id, params, socket_addr).await;
                                    restore_terminal().unwrap();
                                })
                                .await
                            })
                            .await;

                        let mut clients = clients.write().await;
                        clients.remove(&client_id);

                        let _ = handle
                            .close(channel)
                            .await
                            .inspect_err(|e| warn!("error closing channel: {e:?}"));
                    });
                });
                Ok(())
            }));
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

impl<T> Drop for AppHandler<T>
where
    T: SshHandler,
{
    fn drop(&mut self) {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let clients = self.clients.clone();
            let client_id = self.client_id;
            handle.spawn(async move {
                clients.write().await.remove(&client_id);
            });
        }
    }
}
