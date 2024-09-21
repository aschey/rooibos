use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use futures_cancel::FutureExt;
use reactive_graph::owner::{provide_context, use_context};
use reactive_graph::signal::{ReadSignal, signal};
use reactive_graph::traits::Set;
use rooibos_runtime::{ServiceContext, spawn_service};
pub use watch_config;
use watch_config::backend::schematic::AppConfig;
use watch_config::schematic::Config;
use watch_config::{ConfigUpdate, ConfigWatcherService, LoadConfig};

#[derive(Clone)]
pub struct ConfigSignal<T>(ReadSignal<ConfigUpdate<Arc<T>>>);

impl<T> Deref for ConfigSignal<T> {
    type Target = ReadSignal<ConfigUpdate<Arc<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ConfigSignal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn provide_config<T: Config + PartialEq + Send + Sync + 'static>(config: AppConfig<T>) {
    let initial = config.reload().unwrap();
    let (config_signal, set_config_signal) = signal(ConfigUpdate {
        old: initial.clone(),
        new: initial,
    });
    let watcher = ConfigWatcherService::new(config);
    let handle = watcher.handle();
    spawn_service((
        "config_watcher",
        move |context: ServiceContext| async move {
            watcher
                .cancel_on(async move { context.cancelled().await })
                .run()
                .await;
            Ok(())
        },
    ));
    let mut subscriber = handle.subscribe();

    spawn_service((
        "config_handler",
        move |context: ServiceContext| async move {
            while let Ok(Ok(update)) = subscriber.recv().cancel_with(context.cancelled()).await {
                if let Ok(update) = update {
                    set_config_signal.set(update);
                }
            }

            Ok(())
        },
    ));

    provide_context(ConfigSignal(config_signal))
}

pub fn use_config<T: Clone + 'static>() -> ConfigSignal<T> {
    use_context::<ConfigSignal<T>>().unwrap()
}
