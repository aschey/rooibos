use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use ratatui::style::Stylize;
use ratatui::text::Text;
use rooibos_reactive::dom::Render;
use rooibos_reactive::dom::layout::{
    Borders, Dimension, align_items, auto, borders, clear, end, full, height, max_width,
    padding_right, padding_top, width, z_index,
};
use rooibos_reactive::graph::owner::{StoredValue, provide_context, use_context};
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, Update, WithValue};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_reactive::{col, for_each, wgt};
use tokio::sync::broadcast;
use wasm_compat::futures::{sleep, spawn};

#[derive(Clone, Debug)]
pub struct NotificationContext {
    tx: broadcast::Sender<Notification>,
}

impl NotificationContext {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Self { tx }
    }
}

static NOTIFICATION_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Debug)]
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

fn get_notification_context() -> NotificationContext {
    let context = use_context::<NotificationContext>();
    if let Some(context) = context {
        context
    } else {
        let context = NotificationContext::new();
        provide_context(context.clone());
        context
    }
}

pub fn use_notification_context() -> Option<NotificationContext> {
    use_context::<NotificationContext>()
}

#[derive(Clone, Copy, Debug)]
pub struct Notifier {
    context: StoredValue<NotificationContext>,
}

impl Notifier {
    pub fn new(context: NotificationContext) -> Self {
        Self {
            context: StoredValue::new(context),
        }
    }

    pub fn notify(&self, notification: Notification) {
        self.context
            .with_value(|v| v.tx.send(notification).unwrap());
    }
}

pub struct Notifications {
    content_width: Signal<Dimension>,
    max_layout_width: Signal<Dimension>,
    rx: broadcast::Receiver<Notification>,
}

pub fn use_notifications() -> (Notifications, Notifier) {
    let context = get_notification_context();
    (Notifications::new(context.clone()), Notifier::new(context))
}

impl Notifications {
    fn new(context: NotificationContext) -> Self {
        Self {
            content_width: auto().into(),
            max_layout_width: auto().into(),
            rx: context.tx.subscribe(),
        }
    }

    pub fn content_width<S>(mut self, content_width: S) -> Self
    where
        S: Into<Signal<Dimension>>,
    {
        self.content_width = content_width.into();
        self
    }

    pub fn max_layout_width<S>(mut self, max_layout_width: S) -> Self
    where
        S: Into<Signal<Dimension>>,
    {
        self.max_layout_width = max_layout_width.into();
        self
    }

    pub fn render(self) -> impl Render {
        let Notifications {
            content_width,
            max_layout_width,
            mut rx,
        } = self;
        let notifications: RwSignal<Vec<Notification>> = RwSignal::new(vec![]);

        spawn(async move {
            while let Ok(notification) = rx.recv().await {
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
                width(full()),
                height(full()),
                max_width(max_layout_width),
                padding_right(1),
                padding_top(1),
                align_items(end()),
            ),
            col![
                props(width(content_width)),
                for_each(
                    move || notifications.get(),
                    |n| n.id,
                    move |n| {
                        wgt!(
                            props(borders(Borders::all().round().blue()), clear(true)),
                            n.content.clone()
                        )
                    }
                )
            ]
        ]
    }
}
