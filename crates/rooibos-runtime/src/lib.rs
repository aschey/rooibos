use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures_util::StreamExt;
use rooibos_dom::{focused_node, NodeId};
use rooibos_reactive::{
    create_effect, create_memo, create_runtime, create_signal, on_cleanup, provide_context,
    use_context, Effect, ReadSignal, RuntimeId, SignalGet, SignalSet, SignalSetUntracked,
    WriteSignal,
};
use tokio::sync::mpsc;

pub enum Event {
    TermEvent(crossterm::event::Event),
    CancellationComplete(Option<String>),
    QuitRequested,
}

async fn read_input(event_tx: mpsc::Sender<Event>) {
    let mut event_reader = crossterm::event::EventStream::new().fuse();
    while let Some(Ok(event)) = event_reader.next().await {
        if let event::Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                event_tx.send(Event::QuitRequested).await.unwrap();
                break;
            }
            event_tx.send(Event::TermEvent(event)).await.unwrap();
        }
    }
}

#[derive(Clone, Copy)]
struct TermSignal(ReadSignal<Option<crossterm::event::Event>>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TickResult {
    Continue,
    Exit,
}

pub struct Runtime {
    event_rx: mpsc::Receiver<Event>,
    set_last_term_event: WriteSignal<Option<crossterm::event::Event>>,
    _runtime_id: RuntimeId,
}

impl Runtime {
    pub fn initialize() -> Self {
        let _runtime_id = create_runtime();
        let (event_tx, event_rx) = mpsc::channel(32);
        let (last_term_event, set_last_term_event) = create_signal(None);
        provide_context(TermSignal(last_term_event));
        tokio::spawn(async move { read_input(event_tx).await });
        Self {
            _runtime_id,
            event_rx,
            set_last_term_event,
        }
    }

    pub async fn tick(&mut self) -> TickResult {
        match self.event_rx.recv().await {
            Some(Event::TermEvent(e)) => {
                self.set_last_term_event.set(Some(e));
                self.set_last_term_event.set_untracked(None);
            }
            Some(Event::QuitRequested) => {
                return TickResult::Exit;
            }
            _ => {}
        }

        TickResult::Continue
    }
}

pub fn create_key_effect<T>(f: impl Fn(KeyEvent) -> T + 'static) -> Effect<()> {
    let last_term_event = use_context::<TermSignal>().unwrap();
    create_effect(move |_| {
        if let Some(crossterm::event::Event::Key(key_event)) = last_term_event.0.get() {
            if key_event.kind == KeyEventKind::Press {
                f(key_event);
            }
        }
    })
}

pub fn use_focus() -> (NodeId, impl SignalGet<Value = bool> + Copy) {
    let id = NodeId::new_auto();
    use_focus_with_id_inner(id)
}

pub fn use_focus_with_id(id: impl Into<String>) -> (NodeId, impl SignalGet<Value = bool> + Copy) {
    let id = NodeId::new(id);
    use_focus_with_id_inner(id)
}

fn use_focus_with_id_inner(id: NodeId) -> (NodeId, impl SignalGet<Value = bool> + Copy) {
    let id_ = id.clone();

    let focused_node = focused_node();
    let focused = create_memo(move |_| focused_node.get().map(|node| node == id_).unwrap_or(false));

    (id, focused)
}
