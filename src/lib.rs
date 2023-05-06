use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    future,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};

use async_recursion::async_recursion;
use dyn_clonable::clonable;
use futures_util::{stream::FuturesUnordered, Future, Stream};
use leptos_reactive::{
    create_effect, create_runtime, create_scope, create_signal, provide_context, use_context,
    ReadSignal, Scope, ScopeDisposer, SignalGet, SignalSet, WriteSignal,
};
use tokio::{
    runtime::Handle,
    sync::mpsc,
    task::{self, JoinError, JoinHandle},
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub use leptos_reactive as reactive;

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("{0}")]
    SendFailure(String),
    #[error("{0}")]
    JoinFailure(JoinError),
}

#[clonable]
pub trait AnyClone: Any + Clone {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AnyClone for T
where
    T: Any + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub enum Event {
    #[cfg(feature = "crossterm")]
    TermEvent(crossterm::event::Event),
    CancellationComplete(Option<String>),
    QuitRequested,
}

pub enum Request {
    Batch(Vec<Command>),
    Sequence(Vec<Command>),
    Stream(Pin<Box<dyn Stream<Item = Request> + Send>>),
    Quit,
    CancelAll,
    Cancel(String),
    Custom(Box<dyn AnyClone + Send>),
}

#[derive(Debug)]
pub struct Command {
    name: String,
    func: CommandFn,
}

impl Command {
    pub fn new_async<F: Future<Output = Option<Request>> + Send + 'static>(
        f: impl FnOnce(mpsc::Sender<Command>, CancellationToken) -> F + Send + 'static,
    ) -> Self {
        Self {
            name: "".to_owned(),
            func: CommandFn::Async(Box::new(|sender, cancellation_token| {
                Box::pin(async move { f(sender, cancellation_token).await })
            })),
        }
    }

    pub fn new_blocking(
        f: impl FnOnce(mpsc::Sender<Command>, CancellationToken) -> Option<Request> + Send + 'static,
    ) -> Self {
        Self {
            name: "".to_owned(),
            func: CommandFn::Blocking(Box::new(f)),
        }
    }

    pub fn simple(msg: Request) -> Self {
        Self::new_async(|_, _| future::ready(Some(msg)))
    }

    pub fn quit() -> Self {
        Self::simple(Request::Quit)
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            func: self.func,
        }
    }
}

pub type AsyncCommand = dyn FnOnce(
        mpsc::Sender<Command>,
        CancellationToken,
    ) -> Pin<Box<dyn Future<Output = Option<Request>> + Send>>
    + Send;

pub type BlockingCommand =
    dyn FnOnce(mpsc::Sender<Command>, CancellationToken) -> Option<Request> + Send;

pub enum CommandFn {
    Async(Box<AsyncCommand>),
    Blocking(Box<BlockingCommand>),
}

impl core::fmt::Debug for CommandFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Async(_) => f.debug_tuple("Async").field(&"Fn").finish(),
            Self::Blocking(_) => f.debug_tuple("Blocking").field(&"Fn").finish(),
        }
    }
}

#[derive(Clone)]
pub struct EventProvider {
    event_signal: ReadSignal<Option<Event>>,
    custom_signal: ReadSignal<Option<Box<dyn AnyClone + Send>>>,
    command_sender: mpsc::Sender<Command>,
}

impl EventProvider {
    pub fn create_custom_event_signal<T: PartialEq + Sized + Clone + 'static>(
        &self,
    ) -> impl Fn() -> Option<T> {
        let custom_signal = self.custom_signal;

        move || {
            let signal = custom_signal.get();
            signal.as_any().downcast_ref::<T>().cloned()
        }
    }

    pub fn create_event_signal(&self) -> impl Fn() -> Option<Event> {
        let event_signal = self.event_signal;
        move || event_signal.get()
    }

    pub fn disptach(&self, command: Command) {
        self.command_sender.try_send(command).unwrap();
    }
}

pub fn run_system<F, E>(f: F) -> (Result<(), E>, ScopeDisposer)
where
    F: FnOnce(Scope) -> Result<(), E> + 'static,
    E: 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let scope = create_scope(create_runtime(), move |cx| {
        tx.send(f(cx)).unwrap();
    });
    (rx.recv().unwrap(), scope)
}

pub fn use_event_provider(cx: Scope) -> EventProvider {
    use_context::<EventProvider>(cx).unwrap()
}

pub struct EventHandler<W> {
    cx: Scope,
    writer: Rc<RefCell<Option<W>>>,
    set_event: WriteSignal<Option<Event>>,
    cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    set_custom_signal: WriteSignal<Option<Box<dyn AnyClone + Send>>>,
    message_handler_task: Option<JoinHandle<Result<(), MessageError>>>,
    custom_event_rx: mpsc::Receiver<Box<dyn AnyClone + Send>>,
    event_rx: mpsc::Receiver<Event>,
    handler_token: CancellationToken,
}

impl<W: 'static> EventHandler<W> {
    pub fn initialize(cx: Scope, writer: W) -> Self {
        let (read_event_signal, set_event_signal) = create_signal(cx, None);
        let (read_custom_signal, set_custom_signal) = create_signal(cx, None);

        let (command_tx, command_rx) = mpsc::channel(32);
        let (custom_event_tx, custom_event_rx) = mpsc::channel(32);
        let (event_tx, event_rx) = mpsc::channel(32);

        provide_context(
            cx,
            EventProvider {
                event_signal: read_event_signal,
                custom_signal: read_custom_signal,
                command_sender: command_tx.clone(),
            },
        );

        let writer = Rc::new(RefCell::new(Some(writer)));

        let cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>> =
            Default::default();
        let cancellation_token = CancellationToken::default();
        let message_handler_task = MessageHandler::spawn(
            event_tx,
            custom_event_tx,
            command_tx,
            command_rx,
            cancellation_tokens.clone(),
            cancellation_token.clone(),
        );

        Self {
            cx,
            writer,
            cancellation_tokens,
            set_event: set_event_signal,
            set_custom_signal,
            message_handler_task,
            custom_event_rx,
            event_rx,
            handler_token: cancellation_token,
        }
    }

    pub async fn shutdown(&self) {
        for token in self.cancellation_tokens.lock().unwrap().values() {
            token.cancel();
        }
    }

    pub fn render(&self, f: impl Fn(&mut W) + 'static) {
        let writer = self.writer.clone();
        create_effect(self.cx, move |_| {
            if let Some(writer) = writer.borrow_mut().as_mut() {
                f(writer)
            }
        });
    }

    pub async fn run(mut self) -> W {
        #[cfg(feature = "crossterm")]
        let mut event_reader = crossterm::event::EventStream::new().fuse();

        loop {
            #[cfg(feature = "crossterm")]
            let next_event = event_reader.next();
            #[cfg(not(feature = "crossterm"))]
            let next_event = future::pending::<Option<Result<bool, ()>>>();
            tokio::select! {
                // cfg checks don't work with select: https://github.com/tokio-rs/tokio/issues/3974
                Some(Ok(event)) = next_event => {
                    #[cfg(feature = "crossterm")]
                    {
                        use crossterm::event::{KeyModifiers, KeyCode, KeyEvent};

                        if let crossterm::event::Event::Key(KeyEvent {code, modifiers, ..})=  event {
                            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                                self.set_event.set(Some(Event::QuitRequested));
                                break;
                            }
                        }
                        self.set_event.set(Some(Event::TermEvent(event)));
                    }
                    #[cfg(not(feature="crossterm"))]
                    {
                         // suppress unused warnings
                        _ = event;
                    }

                }
                Some(event) = self.event_rx.recv() => {
                    if let Event::QuitRequested = event {
                        self.set_event.set(Some(event));
                        break;
                    } else {
                        self.set_event.set(Some(event));
                    }
                }
                Some(event) = self.custom_event_rx.recv() => {
                    self.set_custom_signal.set(Some(event));
                }
            }
        }

        self.handler_token.cancel();
        if let Some(handler) = self.message_handler_task.take() {
            handler.await.unwrap().unwrap();
        }

        self.writer.borrow_mut().take().unwrap()
    }
}

#[derive(Default, Clone)]
struct FuturesUnorderedCounter {
    futures: Arc<tokio::sync::Mutex<FuturesUnordered<JoinHandle<Result<(), MessageError>>>>>,
    count: Arc<AtomicU32>,
}

impl FuturesUnorderedCounter {
    async fn push(&mut self, future: JoinHandle<Result<(), MessageError>>) {
        self.futures.lock().await.push(future);
        self.count.fetch_add(1, Ordering::SeqCst);
    }

    async fn next(&mut self) -> Option<Result<Result<(), MessageError>, JoinError>> {
        let next = self.futures.lock().await.next().await;
        if next.is_some() {
            self.count.fetch_sub(1, Ordering::SeqCst);
        }
        next
    }

    fn is_empty(&self) -> bool {
        self.count.load(Ordering::SeqCst) == 0
    }
}

#[derive(Clone)]
struct MessageHandler {
    event_tx: mpsc::Sender<Event>,
    custom_event_tx: mpsc::Sender<Box<dyn AnyClone + Send>>,
    command_tx: mpsc::Sender<Command>,
    cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    futs: FuturesUnorderedCounter,
}

impl MessageHandler {
    fn spawn(
        event_tx: mpsc::Sender<Event>,
        custom_event_tx: mpsc::Sender<Box<dyn AnyClone + Send>>,
        command_tx: mpsc::Sender<Command>,
        mut command_rx: mpsc::Receiver<Command>,
        cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
        cancellation_token: CancellationToken,
    ) -> Option<JoinHandle<Result<(), MessageError>>> {
        let mut handler = MessageHandler {
            event_tx,
            custom_event_tx,
            command_tx,
            cancellation_tokens,
            futs: Default::default(),
        };
        Some(tokio::task::spawn(async move {
            loop {
                tokio::select! {
                    Some(cmd) = command_rx.recv() => {
                        {
                            let mut cancellation_tokens = handler.cancellation_tokens.lock().unwrap();
                                if !cancellation_tokens.contains_key(&cmd.name) {
                                    cancellation_tokens.insert(cmd.name.clone(), CancellationToken::new());
                                }
                        }
                        handler.handle_cmd(
                            cmd
                        ).await?;
                    },
                    Some(fut) = handler.futs.next() => {
                        fut.map_err(MessageError::JoinFailure)??;
                        if cancellation_token.is_cancelled() && handler.futs.is_empty() {
                            break;
                        }
                    },
                    _ = cancellation_token.cancelled() => {
                        if handler.futs.is_empty() {
                            break;
                        }
                    }

                }
            }
            Ok(())
        }))
    }

    async fn handle_cmd(&mut self, cmd: Command) -> Result<(), MessageError> {
        let cancellation_token = self
            .cancellation_tokens
            .lock()
            .unwrap()
            .get(&cmd.name)
            .unwrap()
            .clone();
        match cmd.func {
            CommandFn::Async(cmd) => {
                let next = self.clone();
                self.futs
                    .push(tokio::task::spawn(async move {
                        let request = cmd(next.command_tx.clone(), cancellation_token).await;
                        next.handle_request(request).await
                    }))
                    .await;
            }
            CommandFn::Blocking(cmd) => {
                let next = self.clone();
                self.futs
                    .push(tokio::task::spawn_blocking(move || {
                        let request = cmd(next.command_tx.clone(), cancellation_token);
                        let handle: JoinHandle<Result<(), MessageError>> =
                            tokio::task::spawn(async move {
                                next.handle_request(request).await?;
                                Ok(())
                            });
                        Handle::current()
                            .block_on(handle)
                            .map_err(MessageError::JoinFailure)??;
                        Ok(())
                    }))
                    .await;
            }
        }
        Ok(())
    }

    #[async_recursion]
    async fn handle_request(&self, request: Option<Request>) -> Result<(), MessageError> {
        let mut futs = FuturesUnordered::<task::JoinHandle<Result<(), MessageError>>>::default();
        match request {
            Some(Request::Batch(cmds)) => {
                for cmd in cmds {
                    self.command_tx
                        .send(cmd)
                        .await
                        .map_err(|e| MessageError::SendFailure(e.to_string()))?;
                }
            }
            Some(Request::Sequence(cmds)) => {
                let next = self.clone();
                futs.push(tokio::task::spawn(async move {
                    next.handle_sequence_cmd(cmds).await
                }));
            }
            Some(Request::Stream(mut rx)) => {
                let next = self.clone();
                futs.push(task::spawn(async move {
                    while let Some(request) = rx.next().await {
                        let res = next.handle_request(Some(request)).await;
                        res?;
                    }
                    Ok(())
                }));
            }
            Some(Request::CancelAll) => {
                for token in self.cancellation_tokens.lock().unwrap().values() {
                    token.cancel();
                }
                self.event_tx
                    .send(Event::CancellationComplete(None))
                    .await
                    .map_err(|e| MessageError::SendFailure(e.to_string()))?;
            }
            Some(Request::Cancel(name)) => {
                if let Some(token) = self.cancellation_tokens.lock().unwrap().get(&name) {
                    token.cancel();
                }
                self.event_tx
                    .send(Event::CancellationComplete(Some(name)))
                    .await
                    .map_err(|e| MessageError::SendFailure(e.to_string()))?;
            }
            Some(Request::Quit) => {
                self.event_tx
                    .send(Event::QuitRequested)
                    .await
                    .map_err(|e| MessageError::SendFailure(e.to_string()))?;
            }
            Some(Request::Custom(r)) => {
                self.custom_event_tx
                    .send(r)
                    .await
                    .map_err(|e| MessageError::SendFailure(e.to_string()))?;
            }

            None => {}
        }
        while let Some(fut) = futs.next().await {
            fut.map_err(MessageError::JoinFailure)??
        }
        Ok(())
    }

    async fn handle_sequence_cmd(&self, cmds: Vec<Command>) -> Result<(), MessageError> {
        for command in cmds {
            let cancellation_token = self
                .cancellation_tokens
                .lock()
                .unwrap()
                .get(&command.name)
                .unwrap()
                .clone();
            match command.func {
                CommandFn::Async(cmd) => {
                    if let Some(request) = cmd(self.command_tx.clone(), cancellation_token).await {
                        self.command_tx
                            .send(Command::simple(request))
                            .await
                            .map_err(|e| MessageError::SendFailure(e.to_string()))?;
                    }
                }
                CommandFn::Blocking(cmd) => {
                    let command_tx = self.command_tx.clone();
                    let cmd_tx = self.command_tx.clone();
                    let handle: task::JoinHandle<Result<(), MessageError>> =
                        tokio::task::spawn_blocking(move || {
                            if let Some(msg) = cmd(cmd_tx, cancellation_token) {
                                command_tx
                                    .blocking_send(Command::simple(msg))
                                    .map_err(|e| MessageError::SendFailure(e.to_string()))?;
                            }
                            Ok(())
                        });
                    handle.await.map_err(MessageError::JoinFailure)??;
                }
            }
        }
        Ok(())
    }
}
