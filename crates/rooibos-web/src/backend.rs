use std::cell::RefCell;
use std::fmt::Display;
use std::io::{self};
use std::sync::RwLock;

use ratatui::backend::Backend as _;
use ratzilla::ratatui::backend::WindowSize;
use ratzilla::ratatui::layout::Size;
use ratzilla::{CanvasBackend, WebGl2Backend};
use rooibos_dom::Event;
use rooibos_terminal::{AsyncInputStream, ClipboardKind};
use terminput_web_sys::{
    self, to_terminput_key, to_terminput_mouse, to_terminput_mouse_scroll, to_terminput_paste,
    to_terminput_resize,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::prelude::Closure;
use web_sys::{ClipboardEvent, KeyboardEvent, MouseEvent, WheelEvent, window};

pub struct WasmBackend {
    cell_dimensions: RwLock<Size>,
}

impl WasmBackend {
    pub fn new() -> Self {
        Self {
            cell_dimensions: RwLock::new(Size::ZERO),
        }
    }
}

impl Default for WasmBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl rooibos_terminal::Backend for WasmBackend {
    type TuiBackend = WebGl2Backend;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let mut backend = WebGl2Backend::new().unwrap();
        let size = backend.window_size().unwrap();
        let mut cell_dimensions = self.cell_dimensions.write().unwrap();
        cell_dimensions.width = size.pixels.width / size.columns_rows.width;
        cell_dimensions.height = size.pixels.height / size.columns_rows.height;
        Ok(backend)
    }

    fn window_size(&self) -> io::Result<WindowSize> {
        let size = ratzilla::utils::get_window_size();
        Ok(WindowSize {
            columns_rows: size,
            pixels: Size::ZERO,
        })
    }

    fn setup_terminal(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        Ok(())
    }

    fn restore_terminal(&self) -> io::Result<()> {
        Ok(())
    }

    fn enter_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        Ok(())
    }

    fn leave_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        Ok(())
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        backend: &mut Self::TuiBackend,
        title: T,
    ) -> io::Result<()> {
        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        true
    }

    fn set_clipboard<T: Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: ClipboardKind,
    ) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn async_input_stream(&self) -> impl AsyncInputStream {
        let (tx, rx) = mpsc::channel(32);

        let window = window().unwrap();
        let document = window.document().unwrap();
        let cell_dimensions = *self.cell_dimensions.read().unwrap();
        let on_mouse = Closure::<dyn Fn(MouseEvent)>::new({
            let tx = tx.clone();
            move |e: MouseEvent| {
                e.stop_propagation();
                if let Ok(mut mouse_event) = to_terminput_mouse(e) {
                    mouse_event.row /= cell_dimensions.height;
                    mouse_event.column /= cell_dimensions.width;
                    tx.try_send(Event::Mouse(mouse_event)).unwrap();
                }
            }
        });
        let mouse_ref = Some(on_mouse.as_ref().unchecked_ref());
        document.set_onmousedown(mouse_ref);
        document.set_onmouseup(mouse_ref);
        document.set_onmousemove(mouse_ref);

        on_mouse.forget();

        let on_key = Closure::<dyn Fn(KeyboardEvent)>::new({
            let tx = tx.clone();
            move |e: KeyboardEvent| {
                e.stop_propagation();
                if let Ok(key_event) = to_terminput_key(e) {
                    tx.try_send(Event::Key(key_event)).unwrap();
                }
            }
        });
        let key_ref = Some(on_key.as_ref().unchecked_ref());
        document.set_onkeydown(key_ref);
        document.set_onkeyup(key_ref);
        on_key.forget();

        let on_paste = Closure::<dyn Fn(ClipboardEvent)>::new({
            let tx = tx.clone();
            move |e| {
                if let Ok(clipboard_event) = to_terminput_paste(e) {
                    tx.try_send(clipboard_event).unwrap();
                }
            }
        });
        document.set_onpaste(Some(on_paste.as_ref().unchecked_ref()));
        on_paste.forget();

        let on_wheel = Closure::<dyn Fn(WheelEvent)>::new({
            let tx = tx.clone();
            move |e| {
                let scroll_event = to_terminput_mouse_scroll(e);
                tx.try_send(Event::Mouse(scroll_event)).unwrap();
            }
        });
        document.set_onwheel(Some(on_wheel.as_ref().unchecked_ref()));
        on_wheel.forget();

        let on_resize = Closure::<dyn Fn()>::new({
            let window = window.clone();
            let tx = tx.clone();
            move || {
                if let Ok(resize_event) = to_terminput_resize(&window) {
                    tx.try_send(resize_event).unwrap();
                }
            }
        });
        // Note that this only fires when set on the window object
        window.set_onresize(Some(on_resize.as_ref().unchecked_ref()));
        on_resize.forget();

        let on_focus = Closure::<dyn Fn()>::new({
            let tx = tx.clone();
            move || {
                tx.try_send(Event::FocusGained).unwrap();
            }
        });
        window.set_onfocus(Some(on_focus.as_ref().unchecked_ref()));
        on_focus.forget();

        let on_blur = Closure::<dyn Fn()>::new({
            let tx = tx.clone();
            move || {
                tx.try_send(Event::FocusLost).unwrap();
            }
        });
        window.set_onblur(Some(on_blur.as_ref().unchecked_ref()));
        on_blur.forget();

        ReceiverStream::new(rx)
    }
}
