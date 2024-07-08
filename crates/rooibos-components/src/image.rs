use std::thread;

pub use image::io::Reader as ImageReader;
use image::DynamicImage;
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::widgets::StatefulWidget;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::thread::{ThreadImage, ThreadProtocol};
use ratatui_image::{CropOptions, FilterType, Resize};
use reactive_graph::effect::Effect;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Set, Track, Update, UpdateUntracked};
use reactive_graph::wrappers::read::MaybeSignal;
use rooibos_dom::{DomWidget, Render};

#[derive(Clone)]
pub enum ResizeMode {
    Crop(Option<CropOptions>),
    Fit(Option<FilterType>),
}

pub struct Image {
    resize_mode: MaybeSignal<ResizeMode>,
    image: MaybeSignal<DynamicImage>,
}

impl Image {
    pub fn new(image: impl Into<MaybeSignal<DynamicImage>>) -> Self {
        Self {
            image: image.into(),
            resize_mode: ResizeMode::Fit(None).into(),
        }
    }

    pub fn render(self) -> impl Render {
        let Self { resize_mode, image } = self;

        let (tx_worker, rec_worker) =
            std::sync::mpsc::channel::<(Box<dyn StatefulProtocol>, Resize, Rect)>();

        let async_state = RwSignal::new(None);
        Effect::new(move |prev_picker: Option<Picker>| {
            let image = image.get();
            if let Some(mut picker) = prev_picker {
                async_state.set(Some(ThreadProtocol::new(
                    tx_worker.clone(),
                    picker.new_resize_protocol(image),
                )));
                picker
            } else {
                let fallback_size = Size {
                    width: 8,
                    height: 16,
                };
                let mut pixel_size = rooibos_runtime::pixel_size().unwrap_or(fallback_size);
                if pixel_size == Size::default() {
                    pixel_size = fallback_size;
                }
                let mut picker = Picker::new((pixel_size.width, pixel_size.height));
                picker.guess_protocol();

                async_state.set(Some(ThreadProtocol::new(
                    tx_worker.clone(),
                    picker.new_resize_protocol(image),
                )));

                picker
            }
        });

        thread::spawn(move || {
            loop {
                if let Ok((mut protocol, resize, area)) = rec_worker.recv() {
                    protocol.resize_encode(&resize, None, area);
                    async_state.update(|s| {
                        if let Some(s) = s {
                            s.set_protocol(protocol);
                        }
                    });
                }
            }
        });

        DomWidget::new::<ThreadImage, _, _>(move || {
            async_state.track();
            let resize_mode = resize_mode.get();
            move |rect: Rect, buf: &mut Buffer| {
                let image = ThreadImage::default().resize(match resize_mode.clone() {
                    ResizeMode::Crop(options) => Resize::Crop(options),
                    ResizeMode::Fit(filter_type) => Resize::Fit(filter_type),
                });
                async_state.update_untracked(|s| {
                    if let Some(s) = s {
                        image.render(rect, buf, s)
                    }
                });
            }
        })
    }
}
