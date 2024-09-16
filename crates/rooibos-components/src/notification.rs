use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Paragraph};
use rooibos_reactive::div::taffy::AlignItems;
use rooibos_reactive::graph::owner::{provide_context, use_context};
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, Update};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;
use rooibos_reactive::layout::{align_items, chars, clear, height, width, z_index};
use rooibos_reactive::{col, derive_signal, height, wgt, width, Render};
use tokio::sync::mpsc;
use wasm_compat::futures::{sleep, spawn};

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
        let Notifications {
            width: content_width,
        } = self;
        let content_width = derive_signal!(content_width.get() as f32);
        let (tx, mut rx) = mpsc::channel(32);
        provide_context(NotificationContext { tx });

        let notifications: RwSignal<Vec<Notification>> = RwSignal::new(vec![]);

        spawn(async move {
            while let Some(notification) = rx.recv().await {
                let id = notification.id;
                let timeout = notification.timeout;
                notifications.update(|n| n.push(notification));
                spawn(async move {
                    sleep(timeout).await;
                    notifications.update(|n| {
                        let idx = n.iter().position(|n| n.id == id);
                        if let Some(idx) = idx {
                            n.remove(idx);
                        }
                    });
                });
            }
        });

        col![
            props(
                z_index(2),
                width!(100.%),
                height!(100.%),
                align_items(AlignItems::End),
            ),
            col![
                props(width(chars(content_width)),),
                for_each(
                    move || notifications.get(),
                    |n| n.id,
                    move |n| {
                        let content_height = n.content.height() as f32;
                        wgt!(
                            props(
                                // +2 for borders
                                height(chars(content_height + 2.)),
                                clear(true)
                            ),
                            Paragraph::new(n.content.clone()).block(
                                Block::bordered()
                                    .border_type(BorderType::Rounded)
                                    .border_style(Style::new().blue())
                            )
                        )
                    }
                )
            ]
        ]
    }
}
