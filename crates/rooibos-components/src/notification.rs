use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Paragraph};
use reactive_graph::owner::{provide_context, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Update};
use rooibos_dom::{
    absolute, clear, col, derive_signal, use_window_size, widget_ref, Constrainable, Render,
};
use rooibos_runtime::wasm_compat;
use tokio::sync::mpsc;

use crate::for_each;

#[derive(Clone)]
struct NotificationContext {
    tx: mpsc::Sender<Notification>,
}

static NOTIFICATION_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone)]
pub struct Notification {
    id: u32,
    content: Text<'static>,
    timeout: Duration,
}

impl Notification {
    pub fn new(content: impl Into<Text<'static>>) -> Self {
        Self {
            id: NOTIFICATION_ID.fetch_add(1, Ordering::SeqCst),
            content: content.into(),
            timeout: Duration::from_secs(3),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Clone)]
pub struct Notifier {
    context: Arc<RwLock<Option<NotificationContext>>>,
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(use_context::<NotificationContext>())),
        }
    }

    pub fn notify(&self, notification: Notification) {
        if let Some(context) = use_context::<NotificationContext>() {
            context.tx.try_send(notification).unwrap();
            return;
        }

        let context = use_context::<NotificationContext>().unwrap();
        context.tx.try_send(notification).unwrap();
        *self.context.write().unwrap() = Some(context);
    }
}

pub fn notifications() -> impl Render {
    let (tx, mut rx) = mpsc::channel(32);
    provide_context(NotificationContext { tx });

    let notifications: RwSignal<Vec<Notification>> = RwSignal::new(vec![]);
    let window_size = use_window_size();
    let anchor = derive_signal!((window_size.get().width.saturating_sub(20), 0));

    wasm_compat::spawn(async move {
        while let Some(notification) = rx.recv().await {
            let id = notification.id;
            let timeout = notification.timeout;
            notifications.update(|n| n.push(notification));
            wasm_compat::spawn(async move {
                wasm_compat::sleep(timeout).await;
                notifications.update(|n| {
                    let idx = n.iter().position(|n| n.id == id);
                    if let Some(idx) = idx {
                        n.remove(idx);
                    }
                });
            });
        }
    });

    absolute![
        anchor,
        col![for_each(
            move || notifications.get(),
            |n| n.id,
            move |n| {
                let height = n.content.height();
                clear!(widget_ref!(
                    Paragraph::new(n.content.clone()).block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .border_style(Style::new().blue())
                    )
                ))
                // +2 for borders
                .length(height as u16 + 2)
            }
        )]
    ]
}
