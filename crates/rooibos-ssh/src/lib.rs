pub mod backend;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Future;
use reactive_graph::owner::Owner;
use rooibos_dom::Event;
use rooibos_runtime::with_runtime;
pub use russh::server::Config as SshConfig;
use russh::server::{Auth, Handle, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};
pub use russh_keys::key::KeyPair;
use russh_keys::key::PublicKey;
use tokio::net::ToSocketAddrs;
use tokio::sync::{mpsc, RwLock};
use tokio::task::LocalSet;

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
                eprintln!("Failed to send data: {:?}", result);
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
    clients: Arc<RwLock<HashMap<u32, mpsc::Sender<Event>>>>,
    handler: Arc<T>,
    config: Arc<SshConfig>,
    current_client_id: u32,
}

impl<T: SshHandler> AppServer<T> {
    pub fn new(config: SshConfig, handler: T) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            handler: Arc::new(handler),
            config: Arc::new(config),
            current_client_id: 0,
        }
    }

    pub async fn run<A: ToSocketAddrs + Send>(mut self, address: A) -> io::Result<()> {
        self.run_on_address(self.config.clone(), address).await
    }
}

pub trait SshHandler: Send + Sync + 'static {
    fn run_terminal(
        &self,
        client_id: u32,
        handle: ArcHandle,
        event_rx: mpsc::Receiver<Event>,
        client_addr: Option<std::net::SocketAddr>,
    ) -> impl Future + Send;
}

pub struct AppHandler<T>
where
    T: SshHandler,
{
    client_id: u32,
    clients: Arc<RwLock<HashMap<u32, mpsc::Sender<Event>>>>,
    handler: Arc<T>,
    socket_addr: Option<std::net::SocketAddr>,
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
        }
    }
}

#[async_trait]
impl<T: SshHandler> Handler for AppHandler<T> {
    type Error = Box<dyn Error + Send + Sync>;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let (event_tx, event_rx) = mpsc::channel(32);
        self.clients.write().await.insert(self.client_id, event_tx);

        let handler = self.handler.clone();
        let client_id = self.client_id;
        let terminal_handle = TerminalHandle::new(session.handle(), channel.id());
        let socket_addr = self.socket_addr;
        tokio::task::spawn_blocking(move || {
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
                                        event_rx,
                                        socket_addr,
                                    )
                                    .await;
                            })
                            .await
                        })
                        .await;
                })
            });
        });

        Ok(true)
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.eof(channel);
        session.disconnect(russh::Disconnect::ByApplication, "Quit", "");
        session.close(channel);
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
        if let Some(event) = terminput::parser::parse_event(data)? {
            let clients = self.clients.read().await;
            let client_tx = clients.get(&self.client_id).unwrap();
            client_tx.send(event).await.unwrap();
        }

        Ok(())
    }

    /// The client's window size has changed.
    async fn window_change_request(
        &mut self,
        _: ChannelId,
        _col_width: u32,
        _row_height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        // {
        //     let mut clients = self.clients.lock().await;
        //     let terminal = clients.get_mut(&self.id).unwrap();
        //     let rect = Rect {
        //         x: 0,
        //         y: 0,
        //         width: col_width as u16,
        //         height: row_height as u16,
        //     };
        //     terminal.resize(rect)?;
        // }

        Ok(())
    }
}
