use std::ops::{Deref, DerefMut};

use reactive_graph::owner::{provide_context, use_context};
use reactive_graph::signal::{ReadSignal, signal};
use reactive_graph::traits::Set;
use rooibos_runtime::{ServiceContext, spawn_service};
use tokio_util::future::FutureExt;
pub use watch_config;
use watch_config::{ConfigUpdate, ConfigWatcherService, LoadConfig};

pub type ConfigResult<T> =
    Result<ConfigUpdate<<T as LoadConfig>::Config>, <T as LoadConfig>::Error>;

#[derive(Clone)]
pub struct ConfigSignal<T>(ReadSignal<ConfigResult<T>>)
where
    T: LoadConfig;

impl<T> Deref for ConfigSignal<T>
where
    T: LoadConfig,
{
    type Target = ReadSignal<ConfigResult<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ConfigSignal<T>
where
    T: LoadConfig,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn provide_config<T>(config: T)
where
    T: LoadConfig + Send + Sync + 'static,
    T::Config: PartialEq + Clone + Send + Sync + 'static,
    T::Error: Clone + Send + Sync + 'static,
{
    let initial = config.reload();
    let (config_signal, set_config_signal) = signal(initial.map(|i| ConfigUpdate {
        old: i.clone(),
        new: i,
    }));
    let watcher = ConfigWatcherService::new(config);
    let handle = watcher.handle();
    spawn_service((
        "config_watcher",
        move |context: ServiceContext| async move {
            let watcher_token = watcher.cancellation_token().clone();
            let context_token = context.cancellation_token().clone();
            tokio::spawn(async move {
                context_token.cancelled().await;
                watcher_token.cancel();
            });
            watcher.run().await;
            Ok(())
        },
    ));
    let mut subscriber = handle.subscribe();

    spawn_service((
        "config_handler",
        move |context: ServiceContext| async move {
            while let Some(Ok(update)) = subscriber
                .recv()
                .with_cancellation_token(context.cancellation_token())
                .await
            {
                set_config_signal.set(update);
            }

            Ok(())
        },
    ));

    provide_context(ConfigSignal::<T>(config_signal))
}

pub fn use_config<T>() -> ConfigSignal<T>
where
    T: LoadConfig + Clone + 'static,
{
    use_context::<ConfigSignal<T>>().unwrap()
}
