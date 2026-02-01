use std::io;

use rooibos_theme::ColorPalette;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct GuiBackend {}

impl ratatui::backend::Backend for GuiBackend {
    type Error = io::Error;

    fn draw<'a, I>(&mut self, _content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        todo!()
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn get_cursor_position(&mut self) -> Result<ratatui::prelude::Position, Self::Error> {
        todo!()
    }

    fn set_cursor_position<P: Into<ratatui::prelude::Position>>(
        &mut self,
        _position: P,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn clear_region(
        &mut self,
        _clear_type: ratatui::prelude::backend::ClearType,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn size(&self) -> Result<ratatui::prelude::Size, Self::Error> {
        todo!()
    }

    fn window_size(&mut self) -> Result<ratatui::prelude::backend::WindowSize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(
        &mut self,
        _region: core::ops::Range<u16>,
        _line_count: u16,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_down(
        &mut self,
        _region: core::ops::Range<u16>,
        _line_count: u16,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl rooibos_terminal::Backend for GuiBackend {
    type TuiBackend = Self;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        todo!()
    }

    fn setup_terminal(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        todo!()
    }

    fn restore_terminal(&self) -> io::Result<()> {
        todo!()
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        todo!()
    }

    fn enter_alt_screen(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        todo!()
    }

    fn leave_alt_screen(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        todo!()
    }

    fn window_size(
        &self,
        _backend: &mut Self::TuiBackend,
    ) -> io::Result<ratatui::backend::WindowSize> {
        todo!()
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        _backend: &mut Self::TuiBackend,
        _title: T,
    ) -> io::Result<()> {
        todo!()
    }

    fn set_clipboard<T: std::fmt::Display>(
        &self,
        _backend: &mut Self::TuiBackend,
        _content: T,
        _clipboard_kind: rooibos_terminal::ClipboardKind,
    ) -> io::Result<()> {
        todo!()
    }

    fn color_palette(&self) -> ColorPalette {
        todo!()
    }

    fn profile(&self) -> rooibos_theme::TermProfile {
        todo!()
    }

    fn async_input_stream(
        &self,
        _cancellation_token: CancellationToken,
    ) -> impl rooibos_terminal::AsyncInputStream {
        futures::stream::empty()
    }

    fn write_all(&self, _buf: &[u8]) -> io::Result<()> {
        todo!()
    }
}
