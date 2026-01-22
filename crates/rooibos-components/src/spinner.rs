use std::time::Duration;

use ratatui::text::{Line, Span};
use ratatui::widgets::StatefulWidget;
use rooibos_dom::MeasureNode;
use rooibos_dom::widgets::{Role, WidgetRole};
use rooibos_reactive::dom::Render;
use rooibos_reactive::dom::div::taffy::Size;
use rooibos_reactive::graph::IntoReactiveValue;
use rooibos_reactive::graph::computed::Memo;
use rooibos_reactive::graph::effect::Effect;
use rooibos_reactive::graph::owner::on_cleanup;
use rooibos_reactive::graph::signal::{ReadSignal, signal};
use rooibos_reactive::graph::traits::{Get as _, GetUntracked, Update};
use rooibos_reactive::graph::wrappers::read::Signal;
use rooibos_reactive::{IntoSignal, wgt};
use rooibos_theme::Style;
pub use throbber_widgets_tui::WhichUse as SpinnerDisplay;
pub use throbber_widgets_tui::symbols::throbber::*;
use throbber_widgets_tui::{Throbber, ThrobberState};
use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;
use wasm_compat::futures::spawn_local;

#[derive(Clone, Copy)]
pub struct Spinner {
    label: Option<Signal<Span<'static>>>,
    spinner_set: Signal<throbber_widgets_tui::Set>,
    tick_interval: Duration,
    style: Signal<Style>,
    spinner_style: Signal<Style>,
    display: Signal<SpinnerDisplay>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            label: None,
            spinner_set: throbber_widgets_tui::BRAILLE_SIX.into(),
            tick_interval: Duration::from_millis(250),
            style: Style::default().into(),
            spinner_style: Style::default().into(),
            display: SpinnerDisplay::Spin.into(),
        }
    }

    pub fn label<M>(mut self, label: impl IntoReactiveValue<Signal<Span<'static>>, M>) -> Self {
        self.label = Some(label.into_reactive_value());
        self
    }

    pub fn spinner_set<M>(
        mut self,
        spinner_set: impl IntoReactiveValue<Signal<throbber_widgets_tui::Set>, M>,
    ) -> Self {
        self.spinner_set = spinner_set.into_reactive_value();
        self
    }

    pub fn tick_interval(mut self, tick_interval: Duration) -> Self {
        self.tick_interval = tick_interval;
        self
    }

    pub fn style<M>(mut self, style: impl IntoReactiveValue<Signal<Style>, M>) -> Self {
        self.style = style.into_reactive_value();
        self
    }

    pub fn spinner_style<M>(
        mut self,
        spinner_style: impl IntoReactiveValue<Signal<Style>, M>,
    ) -> Self {
        self.spinner_style = spinner_style.into_reactive_value();
        self
    }

    pub fn display<M>(
        mut self,
        display: impl IntoReactiveValue<Signal<SpinnerDisplay>, M>,
    ) -> Self {
        self.display = display.into_reactive_value();
        self
    }

    fn create_state(&self) -> ReadSignal<ThrobberState> {
        let Self {
            tick_interval,
            display,
            ..
        } = self;

        let (state, set_state) = signal(ThrobberState::default());
        let spinner_active = Memo::new({
            let display = *display;
            move |_| matches!(display.get(), SpinnerDisplay::Spin)
        });
        let tick_interval = *tick_interval;

        let cancellation_token = CancellationToken::new();
        {
            let cancellation_token = cancellation_token.clone();
            Effect::new(move |child_token: Option<CancellationToken>| {
                if spinner_active.get() {
                    let child_token = cancellation_token.child_token();
                    spawn_local({
                        let child_token = child_token.clone();
                        async move {
                            loop {
                                if wasm_compat::futures::sleep(tick_interval)
                                    .with_cancellation_token(&child_token)
                                    .await
                                    .is_none()
                                {
                                    return;
                                }

                                if set_state.try_update(|s| s.calc_next()).is_none() {
                                    return;
                                }
                            }
                        }
                    });
                    child_token
                } else if let Some(child_token) = child_token {
                    child_token.cancel();
                    child_token
                } else {
                    cancellation_token.child_token()
                }
            });
        };

        on_cleanup(move || {
            cancellation_token.cancel();
        });

        state
    }

    fn create_spinner_fn(self) -> impl Fn() -> Throbber<'static> {
        let Self {
            label,
            spinner_set,
            tick_interval: _,
            style,
            spinner_style,
            display,
        } = self;

        move || {
            let mut spinner = Throbber::default();
            if let Some(label) = &label {
                spinner = spinner.label(label.get());
            }

            spinner
                .throbber_set(spinner_set.get())
                .style(style.get().into())
                .throbber_style(spinner_style.get().into())
                .use_type(display.get())
        }
    }

    pub fn into_span_signal(self) -> Signal<Span<'static>> {
        let state = self.create_state();
        let create_spinner_fn = self.create_spinner_fn();
        (move || create_spinner_fn().to_symbol_span(&state.get())).signal()
    }

    pub fn into_line_signal(self) -> Signal<Line<'static>> {
        let state = self.create_state();
        let create_spinner_fn = self.create_spinner_fn();
        (move || create_spinner_fn().to_line(&state.get())).signal()
    }

    pub fn render(self) -> impl Render {
        let label = self.label;
        let spinner_set = self.spinner_set;
        let state = self.create_state();
        let create_spinner = self.create_spinner_fn();

        wgt!(
            state.get(),
            SpinnerWidget {
                inner: create_spinner(),
                label,
                spinner_set
            }
        )
    }
}

#[derive(Clone)]
struct SpinnerWidget<'a> {
    inner: Throbber<'a>,
    label: Option<Signal<Span<'static>>>,
    spinner_set: Signal<throbber_widgets_tui::Set>,
}

impl WidgetRole for SpinnerWidget<'_> {
    fn widget_role() -> Option<Role> {
        None
    }
}

impl SpinnerWidget<'_> {
    fn spinner_size(&self) -> Size<f32> {
        let set = self.spinner_set.get_untracked();
        let w_max = set
            .symbols
            .iter()
            .map(|s| s.estimate_size().width)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.);
        let h_max = set
            .symbols
            .iter()
            .map(|s| s.estimate_size().height)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.);

        set.empty
            .estimate_size()
            .f32_max(set.full.estimate_size())
            .f32_max(Size {
                width: w_max,
                height: h_max,
            })
    }
}

impl MeasureNode for SpinnerWidget<'_> {
    fn measure(
        &self,
        _known_dimensions: rooibos_reactive::dom::div::taffy::Size<Option<f32>>,
        _available_space: rooibos_reactive::dom::div::taffy::Size<
            rooibos_reactive::dom::div::taffy::AvailableSpace,
        >,
        _style: &rooibos_reactive::dom::div::taffy::Style,
    ) -> rooibos_reactive::dom::div::taffy::Size<f32> {
        self.estimate_size()
    }

    fn estimate_size(&self) -> Size<f32> {
        let label_size = self
            .label
            .as_ref()
            .map(|l| l.get().estimate_size())
            .unwrap_or_default();
        let spinner_size = self.spinner_size();
        Size {
            width: label_size.width + spinner_size.width,
            height: label_size.height.max(spinner_size.height),
        }
    }
}

impl StatefulWidget for SpinnerWidget<'_> {
    type State = ThrobberState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        self.inner.render(area, buf, state)
    }
}
