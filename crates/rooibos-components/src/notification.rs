use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use ratatui::style::Stylize;
use ratatui::text::Text;
use rooibos_reactive::dom::Render;
use rooibos_reactive::dom::div::taffy::{self, AlignItems};
use rooibos_reactive::dom::layout::{
    Borders, align_items, borders, clear, max_width, width, z_index,
};
use rooibos_reactive::graph::owner::{StoredValue, provide_context, use_context};
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, Update, WithValue};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;
use rooibos_reactive::{col, for_each, height, wgt, width};
use tokio::sync::broadcast;
use wasm_compat::futures::{sleep, spawn};

#[derive(Clone, Debug)]
pub struct NotificationContext {
    tx: broadcast::Sender<Notification>,
}

impl Default for NotificationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationContext {
    pub fn new() -> Self {
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

pub fn provide_notifications() {
    provide_context(NotificationContext::new())
}

fn get_notification_context() -> NotificationContext {
    use_context::<NotificationContext>().expect(
        "Notification context not found. Ensure provide_notifications() was called at the root of \
         your application.",
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Notifier {
    context: StoredValue<NotificationContext>,
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            context: StoredValue::new(get_notification_context()),
        }
    }

    pub fn notify(&self, notification: Notification) {
        self.context
            .with_value(|v| v.tx.send(notification).unwrap());
    }
}

pub struct Notifications {
    content_width: MaybeSignal<taffy::Dimension>,
    max_layout_width: MaybeSignal<taffy::Dimension>,
    rx: broadcast::Receiver<Notification>,
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    pub fn new() -> Self {
        let context = get_notification_context();
        Self {
            content_width: taffy::Dimension::Auto.into(),
            max_layout_width: taffy::Dimension::Auto.into(),
            rx: context.tx.subscribe(),
        }
    }

    pub fn content_width<S>(mut self, content_width: S) -> Self
    where
        S: Into<MaybeSignal<taffy::Dimension>>,
    {
        self.content_width = content_width.into();
        self
    }

    pub fn max_layout_width<S>(mut self, max_layout_width: S) -> Self
    where
        S: Into<MaybeSignal<taffy::Dimension>>,
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
                width!(100.%),
                height!(100.%),
                max_width(max_layout_width),
                align_items(AlignItems::End),
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
