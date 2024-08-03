use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Paragraph};
use reactive_graph::owner::{provide_context, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Update};
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{
    absolute, clear, col, derive_signal, length, props, use_window_size, widget_ref, Constrainable,
    Render,
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

pub struct Notifications {
    width: MaybeSignal<u16>,
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    pub fn new() -> Self {
        Self { width: 20.into() }
    }

    pub fn width<S>(mut self, width: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        self.width = width.into();
        self
    }

    pub fn render(self) -> impl Render {
        let Notifications { width } = self;
        let (tx, mut rx) = mpsc::channel(32);
        provide_context(NotificationContext { tx });

        let notifications: RwSignal<Vec<Notification>> = RwSignal::new(vec![]);
        let window_size = use_window_size();
        let anchor = derive_signal!((window_size.get().width.saturating_sub(width.get()), 0));

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
                    clear![
                         // +2 for borders
                        props!(length(height as u16 + 2));
                        widget_ref!(
                            Paragraph::new(n.content.clone()).block(
                                Block::bordered()
                                    .border_type(BorderType::Rounded)
                                    .border_style(Style::new().blue())
                            )
                    )]
                    .length(height as u16 + 2)
                }
            )]
        ]
    }
}
