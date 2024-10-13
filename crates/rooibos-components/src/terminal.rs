use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

pub use portable_pty::CommandBuilder;
use portable_pty::{MasterPty, NativePtySystem, PtySize, PtySystem, SlavePty};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Widget};
use rooibos_dom::Event;
use rooibos_reactive::graph::owner::StoredValue;
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, Update};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;
use rooibos_reactive::{DomWidget, Render};
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

#[derive(Clone, Copy)]
pub struct TerminalRef {
    master: StoredValue<Arc<Mutex<Box<dyn MasterPty + Send>>>>,
    slave: StoredValue<Arc<Mutex<Box<dyn SlavePty + Send>>>>,
}

impl TerminalRef {
    pub fn spawn_command(&self, command: CommandBuilder) {
        let mut child = self
            .slave
            .get_value()
            .lock()
            .unwrap()
            .spawn_command(command)
            .unwrap();

        spawn_blocking(move || {
            child.wait().unwrap();
        });
    }
}

#[derive(Default)]
pub struct Terminal {
    block: Option<MaybeSignal<Block<'static>>>,
}

impl Terminal {
    pub fn get_ref() -> TerminalRef {
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: 1,
                cols: 1,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        let master = StoredValue::new(Arc::new(Mutex::new(pair.master)));
        let slave = StoredValue::new(Arc::new(Mutex::new(pair.slave)));

        TerminalRef { master, slave }
    }

    pub fn block(mut self, block: impl Into<MaybeSignal<Block<'static>>>) -> Self {
        self.block = Some(block.into());
        self
    }

    pub fn render(self, terminal_ref: TerminalRef) -> impl Render {
        let Self { block } = self;
        let border_size = if block.is_some() { 1 } else { 0 };
        let TerminalRef { master, .. } = terminal_ref;
        let parser = RwSignal::new(Arc::new(Mutex::new(vt100::Parser::new(1, 1, 0))));

        let master_ = master.get_value();
        let master_ = master_.lock().unwrap();
        let mut reader = master_.try_clone_reader().unwrap();
        let mut writer = BufWriter::new(master_.take_writer().unwrap());
        drop(master_);

        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);
        tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                writer.write_all(&bytes).unwrap();
                writer.flush().unwrap();
            }
        });

        std::thread::spawn(move || {
            // Consume the output from the child
            // Can't read the full buffer, since that would wait for EOF
            let mut buf = [0u8; 8192];
            let mut processed_buf = Vec::new();
            loop {
                let size = reader.read(&mut buf).unwrap();
                if size == 0 {
                    break;
                }
                if size > 0 {
                    processed_buf.extend_from_slice(&buf[..size]);
                    parser.update(|p| {
                        let mut parser = p.lock().unwrap();
                        parser.process(&processed_buf);
                    });

                    // Clear the processed portion of the buffer
                    processed_buf.clear();
                }
            }
        });

        DomWidget::new::<PseudoTerminal<Screen>, _, _>(move || {
            let parser = parser.get();
            let block = block.as_ref().map(|b| b.get());
            move |rect: Rect, frame: &mut Frame| {
                let parser = parser.lock().unwrap();

                let mut term = PseudoTerminal::new(parser.screen());
                if let Some(block) = block.clone() {
                    term = term.block(block);
                }
                term.render(rect, frame.buffer_mut());
            }
        })
        .on_key_down(move |key, _, _| {
            tx.try_send(Event::Key(key).to_escape_sequence()).unwrap();
        })
        .on_size_change(move |rect| {
            master
                .get_value()
                .lock()
                .unwrap()
                .resize(PtySize {
                    rows: rect.height - border_size,
                    cols: rect.width - border_size,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .unwrap();
            parser.update(|p| {
                p.lock().unwrap().set_size(rect.height, rect.width);
            })
        })
    }
}
