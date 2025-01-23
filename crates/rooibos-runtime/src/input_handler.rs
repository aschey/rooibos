use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use background_service::ServiceContext;
use rooibos_dom::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use tokio::sync::broadcast;
use tracing::{error, warn};
use wasm_compat::sync::Mutex;

use crate::debounce::Debouncer;
use crate::{EventFilter, InputMode, IsQuitEvent, RuntimeCommand, has_external_signal_stream};

pub(crate) struct InputHandler {
    pub(crate) term_parser_rx: broadcast::Receiver<Event>,
    pub(crate) signal_tx: broadcast::Sender<RuntimeCommand>,
    pub(crate) term_event_tx: broadcast::Sender<Event>,
    pub(crate) hover_debouncer: Debouncer<Event>,
    pub(crate) resize_debouncer: Debouncer<Event>,
    pub(crate) context: ServiceContext,
    pub(crate) is_quit_event: Arc<IsQuitEvent>,
    pub(crate) editing: Arc<AtomicBool>,
    pub(crate) event_filter: Arc<Mutex<Box<EventFilter>>>,
}

impl InputHandler {
    pub(crate) async fn handle(&mut self) -> bool {
        tokio::select! {
            next_event = self.term_parser_rx.recv() => {
                if !self.handle_term_event(
                    next_event,

                ).await {
                    return false;
                }
            }
            _ = self.context.cancelled() => {
                return false;
            }
            Some(pending_move) = self.hover_debouncer.next_value() => {
                let _ = self.term_event_tx
                    .send(pending_move)
                    .inspect_err(|e| warn!("error sending mouse move {e:?}"));
            }
            Some(pending_resize) = self.resize_debouncer.next_value() => {
                let _ = self.term_event_tx
                    .send(pending_resize)
                    .inspect_err(|e| warn!("error sending resize event {e:?}"));
            }
        }

        true
    }

    async fn handle_term_event(
        &mut self,
        next_event: Result<Event, broadcast::error::RecvError>,
    ) -> bool {
        let Ok(next_event) = next_event.inspect_err(|e| {
            warn!("event error {e:?}");
        }) else {
            return false;
        };

        let editing = self.editing.load(Ordering::SeqCst);

        let Some(next_event) = self.event_filter.lock_mut()(
            next_event,
            if editing {
                InputMode::Insert
            } else {
                InputMode::Normal
            },
        ) else {
            return true;
        };

        match next_event {
            event @ Event::Key(key_event) => {
                self.handle_key_event(event, key_event, editing);
            }

            mouse_event @ Event::Mouse(MouseEvent {
                kind: MouseEventKind::Moved,
                ..
            }) => {
                let _ = self
                    .hover_debouncer
                    .update(mouse_event)
                    .await
                    .inspect_err(|e| error!("error debouncing hover event: {e:?}"));
            }
            resize_event @ Event::Resize { .. } => {
                let _ = self
                    .resize_debouncer
                    .update(resize_event)
                    .await
                    .inspect_err(|e| error!("error debouncing resize event: {e:?}"));
            }
            event => {
                self.term_event_tx.send(event).ok();
            }
        }
        true
    }

    fn handle_key_event(&self, event: Event, key_event: KeyEvent, editing: bool) {
        let has_modifiers = key_event.modifiers != KeyModifiers::empty();
        // If we're in editing mode, we should always pass through any normal input events (keys
        // with no modifiers) otherwise, it would be impossible to type certain letters
        // ('q' by default)
        if (!editing || has_modifiers) && (self.is_quit_event)(key_event) {
            let _ = self
                .signal_tx
                .send(RuntimeCommand::Terminate(Ok(proc_exit::Code::SUCCESS)))
                .inspect_err(|_| warn!("error sending terminate signal"));
        } else if cfg!(unix)
            && key_event.modifiers == KeyModifiers::CTRL
            && key_event.code == KeyCode::Char('z')
        {
            // Defer to the external stream for suspend commands if it exists
            if !has_external_signal_stream() {
                let _ = self
                    .signal_tx
                    .send(RuntimeCommand::Suspend)
                    .inspect_err(|_| warn!("error sending suspend signal"));
            }
        } else {
            let _ = self
                .term_event_tx
                .send(event)
                .inspect_err(|_| warn!("error sending terminal event"));
        }
    }
}
