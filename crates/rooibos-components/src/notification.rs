use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use once_cell::sync::Lazy;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Update};
use rooibos_dom::{
    absolute, clear, col, derive_signal, use_window_size, widget_ref, Constrainable, Render,
};
use tokio::sync::mpsc;

use crate::for_each;

type NotificationChannel = (
    mpsc::Sender<Notification>,
    Mutex<Option<mpsc::Receiver<Notification>>>,
);

static NOTIFICATION_ID: AtomicU32 = AtomicU32::new(1);

static NOTIFICATION_CHANNEL: Lazy<NotificationChannel> = Lazy::new(move || {
    let (tx, rx) = mpsc::channel(32);
    (tx, Mutex::new(Some(rx)))
});

#[derive(Clone)]
pub struct Notification {
    id: u32,
    content: Text<'static>,
}

impl Notification {
    pub fn new(content: impl Into<Text<'static>>) -> Self {
        Self {
            id: NOTIFICATION_ID.fetch_add(1, Ordering::SeqCst),
            content: content.into(),
        }
    }
}

pub fn notify(notification: Notification) {
    NOTIFICATION_CHANNEL.0.try_send(notification).unwrap();
}

pub fn notifications() -> impl Render {
    let notifications: RwSignal<Vec<Notification>> = RwSignal::new(vec![]);
    let window_size = use_window_size();
    let anchor = derive_signal!((window_size.get().width.saturating_sub(20), 0));

    let Some(mut notification_rx) = NOTIFICATION_CHANNEL.1.lock().unwrap().take() else {
        panic!("notifications already created");
    };

    tokio::spawn(async move {
        while let Some(notification) = notification_rx.recv().await {
            let id = notification.id;
            notifications.update(|n| n.push(notification));
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
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
            move |n| clear!(widget_ref!(
                Paragraph::new(n.content.clone()).block(Block::bordered())
            ))
            .length(3)
        )]
    ]
}
