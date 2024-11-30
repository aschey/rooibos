use std::path::PathBuf;
use std::thread;

use image::DynamicImage;
pub use image::ImageReader;
use ratatui::Frame;
use ratatui::layout::{Rect, Size};
use ratatui::widgets::StatefulWidget;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::thread::{ThreadImage, ThreadProtocol};
use ratatui_image::{CropOptions, FilterType, Resize};
use rooibos_dom::{MeasureNode, RenderNode, pixel_size};
use rooibos_reactive::dom::div::taffy;
use rooibos_reactive::dom::{DomWidget, Render};
use rooibos_reactive::graph::effect::Effect;
use rooibos_reactive::graph::signal::RwSignal;
use rooibos_reactive::graph::traits::{Get, Set, Track, Update, UpdateUntracked, With};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;

#[derive(Clone)]
pub enum ResizeMode {
    Crop(Option<CropOptions>),
    Fit(Option<FilterType>),
}

pub struct Image {
    resize_mode: MaybeSignal<ResizeMode>,
    image_source: ImageSource,
}

#[derive(Clone)]
pub enum ImageSource {
    Url(MaybeSignal<PathBuf>),
    Binary(MaybeSignal<DynamicImage>),
}

impl Image {
    pub fn from_url(url: impl Into<MaybeSignal<PathBuf>>) -> Self {
        Self {
            image_source: ImageSource::Url(url.into()),
            resize_mode: ResizeMode::Fit(None).into(),
        }
    }

    pub fn from_binary(binary: impl Into<MaybeSignal<DynamicImage>>) -> Self {
        Self {
            image_source: ImageSource::Binary(binary.into()),
            resize_mode: ResizeMode::Fit(None).into(),
        }
    }

    pub fn render(self) -> impl Render {
        let Self {
            resize_mode,
            image_source,
        } = self;

        let image = RwSignal::new(None);

        Effect::new(move || match &image_source {
            ImageSource::Url(url) => {
                let url = url.get();
                thread::spawn(move || {
                    let decoded = ImageReader::open(url).unwrap().decode().unwrap();
                    image.set(Some(decoded));
                });
            }
            ImageSource::Binary(binary) => {
                image.set(Some(binary.get()));
            }
        });

        let (tx_worker, rec_worker) =
            std::sync::mpsc::channel::<(Box<dyn StatefulProtocol>, Resize, Rect)>();

        let async_state = RwSignal::new(None);
        let fallback_size = Size {
            width: 8,
            height: 16,
        };
        let mut pixel_size = pixel_size().unwrap_or(fallback_size);
        if pixel_size == Size::default() {
            pixel_size = fallback_size;
        }
        Effect::new(move |prev_picker: Option<Option<Picker>>| {
            let image = image.get();
            if let Some(image) = image {
                let (mut picker, image) = if let Some(Some(picker)) = prev_picker {
                    (picker, image)
                } else {
                    let mut picker = Picker::new((pixel_size.width, pixel_size.height));
                    picker.guess_protocol();

                    (picker, image)
                };

                async_state.set(Some(ThreadProtocol::new(
                    tx_worker.clone(),
                    picker.new_resize_protocol(image),
                )));
                Some(picker)
            } else {
                None
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

        DomWidget::new::<ThreadImage, _>(move || {
            async_state.track();
            let image_size = image.with(|i| {
                i.as_ref().map(|i| taffy::Size {
                    width: (i.width() / pixel_size.width as u32) as f32,
                    height: (i.height() / pixel_size.height as u32) as f32,
                })
            });
            RenderImage {
                async_state,
                resize_mode: resize_mode.get(),
                size: image_size.unwrap_or_default(),
            }
        })
    }
}

struct RenderImage {
    async_state: RwSignal<Option<ThreadProtocol>>,
    resize_mode: ResizeMode,
    size: taffy::Size<f32>,
}

impl RenderNode for RenderImage {
    fn render(&mut self, rect: Rect, frame: &mut Frame) {
        let image = ThreadImage::default().resize(match self.resize_mode.clone() {
            ResizeMode::Crop(options) => Resize::Crop(options),
            ResizeMode::Fit(filter_type) => Resize::Fit(filter_type),
        });
        self.async_state.update_untracked(|s| {
            if let Some(s) = s {
                image.render(rect, frame.buffer_mut(), s)
            }
        });
    }
}

impl MeasureNode for RenderImage {
    fn measure(
        &self,
        known_dimensions: rooibos_reactive::dom::div::taffy::Size<Option<f32>>,
        available_space: rooibos_reactive::dom::div::taffy::Size<
            rooibos_reactive::dom::div::taffy::AvailableSpace,
        >,
        style: &rooibos_reactive::dom::div::taffy::Style,
    ) -> rooibos_reactive::dom::div::taffy::Size<f32> {
        taffy::Size {
            width: match available_space.width {
                taffy::AvailableSpace::Definite(s) => s.min(self.size.width),
                _ => self.size.width,
            },
            height: match available_space.height {
                taffy::AvailableSpace::Definite(s) => s.min(self.size.height),
                _ => self.size.height,
            },
        }
    }

    fn estimate_size(&self) -> rooibos_reactive::dom::div::taffy::Size<f32> {
        self.size
    }
}
