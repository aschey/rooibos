use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use background_service::ServiceContext;
use rooibos_dom::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use tokio::sync::broadcast;
use tracing::warn;

use crate::debounce::Debouncer;
use crate::{has_external_signal_stream, IsQuitEvent, RuntimeCommand};

pub(crate) struct InputHandler {
    pub(crate) term_parser_rx: broadcast::Receiver<Event>,
    pub(crate) signal_tx: broadcast::Sender<RuntimeCommand>,
    pub(crate) term_event_tx: broadcast::Sender<Event>,
    pub(crate) hover_debouncer: Debouncer<Event>,
    pub(crate) resize_debouncer: Debouncer<Event>,
    pub(crate) context: ServiceContext,
    pub(crate) is_quit_event: Arc<IsQuitEvent>,
    pub(crate) editing: Arc<AtomicBool>,
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
            pending_move = self.hover_debouncer.next_value() => {
                let _ = self.term_event_tx
                    .send(pending_move)
                    .inspect_err(|e| warn!("error sending mouse move {e:?}"));
            }
            pending_resize = self.resize_debouncer.next_value() => {
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
        match next_event {
            Ok(event @ Event::Key(key_event)) => {
                self.handle_key_event(event, key_event);
            }
            Ok(
                mouse_event @ Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Moved,
                    ..
                }),
            ) => {
                self.hover_debouncer.update(mouse_event).await;
            }
            Ok(resize_event @ Event::Resize(_, _)) => {
                self.resize_debouncer.update(resize_event).await;
            }
            Ok(event) => {
                self.term_event_tx.send(event).ok();
            }
            Err(_) => {
                return false;
            }
        }
        true
    }

    fn handle_key_event(&self, event: Event, key_event: KeyEvent) {
        let editing = self.editing.load(Ordering::SeqCst);
        let has_modifiers = key_event.modifiers != KeyModifiers::empty();
        // If we're in editing mode, we should always pass through any normal input events (keys
        // with no modifiers) otherwise, it would be impossible to type certain letters
        // ('q' by default)
        if (!editing || has_modifiers) && (self.is_quit_event)(key_event) {
            let _ = self
                .signal_tx
                .send(RuntimeCommand::Terminate)
                .inspect_err(|_| warn!("error sending terminate signal"));
        } else if cfg!(unix)
            && key_event.modifiers == KeyModifiers::CONTROL
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
