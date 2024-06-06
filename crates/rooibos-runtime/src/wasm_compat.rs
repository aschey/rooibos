use std::time::Duration;

use any_spawner::Executor;
use futures_util::Future;

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    Executor::spawn(fut)
}

#[cfg(target_arch = "wasm32")]
pub fn spawn(fut: impl Future<Output = ()> + 'static) {
    Executor::spawn_local(fut)
}

pub fn spawn_local(fut: impl Future<Output = ()> + 'static) {
    Executor::spawn_local(fut)
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    let ms = duration.as_millis() as i32;
    use wasm_bindgen_futures::JsFuture;
    let fut: JsFuture = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    })
    .into();
    fut.await.unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await
}

#[cfg(target_arch = "wasm32")]
pub fn now() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64
}

#[cfg(target_arch = "wasm32")]
pub struct Once<T>(::std::cell::OnceCell<T>);

#[cfg(not(target_arch = "wasm32"))]
pub struct Once<T>(std::sync::OnceLock<T>);

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Once<T> {
    pub const fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        return Self(::std::cell::OnceCell::new());
        #[cfg(not(target_arch = "wasm32"))]
        return Self(std::sync::OnceLock::new());
    }

    pub fn set(&self, val: T) -> Result<(), T> {
        self.0.set(val)
    }

    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }
}

#[cfg(target_arch = "wasm32")]
pub struct BoolCell(::std::cell::RefCell<bool>);

#[cfg(not(target_arch = "wasm32"))]
pub struct BoolCell(std::sync::atomic::AtomicBool);

impl BoolCell {
    pub const fn new(val: bool) -> Self {
        #[cfg(target_arch = "wasm32")]
        return Self(::std::cell::RefCell::new(val));
        #[cfg(not(target_arch = "wasm32"))]
        return Self(std::sync::atomic::AtomicBool::new(val));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set(&self, val: bool) {
        *self.0.borrow_mut() = val;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set(&self, val: bool) {
        self.0.store(val, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get(&self) -> bool {
        *self.0.borrow()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Mutex<T>(std::cell::RefCell<T>);

#[cfg(not(target_arch = "wasm32"))]
pub struct Mutex<T>(std::sync::Mutex<T>);

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        #[cfg(target_arch = "wasm32")]
        return Self(std::cell::RefCell::new(val));
        #[cfg(not(target_arch = "wasm32"))]
        return Self(std::sync::Mutex::new(val));
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let inner = self.0.lock().unwrap();
            f(&*inner)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let inner = self.0.borrow();
            f(&*inner)
        }
    }

    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut inner = self.0.lock().unwrap();
            f(&mut *inner)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut inner = self.0.borrow_mut();
            f(&mut *inner)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct LocalKey<T>(T);

#[cfg(not(target_arch = "wasm32"))]
impl<T> LocalKey<T> {
    pub const fn new(val: T) -> Self {
        Self(val)
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.0)
    }
}

#[macro_export]
macro_rules! once {
    // empty (base case for the recursion)
    () => {};

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = const $init:block; $($rest:tt)*) => (
        #[cfg(target_arch="wasm32")]
        thread_local! {
            $vis static $name: $t = const $init;
        }

        #[cfg(not(target_arch="wasm32"))]
        $vis static $name: $crate::wasm_compat::LocalKey<$t> = const $crate::wasm_compat::LocalKey::new($init);

        $crate::once!($($rest)*);
    );

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = const $init:block) => (
        #[cfg(target_arch="wasm32")]
        thread_local! {
            $vis static $name: $t = const $init;
        }

        #[cfg(not(target_arch="wasm32"))]
        $vis static $name: $crate::wasm_compat::LocalKey<$t> = const $crate::wasm_compat::LocalKey::new($init);

        $crate::once!($($rest)*);
    );

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        #[cfg(target_arch="wasm32")]
        thread_local! {
            $vis static $name: $t = $init;
        }

        #[cfg(not(target_arch="wasm32"))]
        $vis static $name: $crate::wasm_compat::LocalKey<$t> = $crate::wasm_compat::LocalKey::new($init);

        $crate::once!($($rest)*);
    );

    // handle a single declaration
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => (
        #[cfg(target_arch="wasm32")]
        thread_local! {
            $vis static $name: $t = const $init;
        }
        #[cfg(not(target_arch="wasm32"))]
        $vis static $name: $crate::wasm_compat::LocalKey<$t> = $crate::wasm_compat::LocalKey::new($init);

        $crate::once!($($rest)*);
    );
}
