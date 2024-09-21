use std::time::Duration;

use futures_cancel::FutureExt as _;
use ratatui::style::Style;
use ratatui::text::Span;
use rooibos_reactive::graph::computed::Memo;
use rooibos_reactive::graph::effect::Effect;
use rooibos_reactive::graph::owner::on_cleanup;
use rooibos_reactive::graph::signal::signal;
use rooibos_reactive::graph::traits::{Get as _, Update};
use rooibos_reactive::graph::wrappers::read::MaybeSignal;
use rooibos_reactive::{Render, derive_signal, wgt};
pub use throbber_widgets_tui::WhichUse as SpinnerDisplay;
pub use throbber_widgets_tui::symbols::throbber::*;
use throbber_widgets_tui::{Throbber, ThrobberState};
use tokio_util::sync::CancellationToken;
use wasm_compat::futures::spawn_local;

pub struct Spinner {
    label: Option<MaybeSignal<Span<'static>>>,
    spinner_set: MaybeSignal<throbber_widgets_tui::Set>,
    tick_interval: Duration,
    style: MaybeSignal<Style>,
    spinner_style: MaybeSignal<Style>,
    display: MaybeSignal<SpinnerDisplay>,
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

    pub fn label(mut self, label: impl Into<MaybeSignal<Span<'static>>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn spinner_set(
        mut self,
        spinner_set: impl Into<MaybeSignal<throbber_widgets_tui::Set>>,
    ) -> Self {
        self.spinner_set = spinner_set.into();
        self
    }

    pub fn tick_interval(mut self, tick_interval: Duration) -> Self {
        self.tick_interval = tick_interval;
        self
    }

    pub fn style(mut self, style: impl Into<MaybeSignal<Style>>) -> Self {
        self.style = style.into();
        self
    }

    pub fn spinner_style(mut self, spinner_style: impl Into<MaybeSignal<Style>>) -> Self {
        self.spinner_style = spinner_style.into();
        self
    }

    pub fn display(mut self, display: impl Into<MaybeSignal<SpinnerDisplay>>) -> Self {
        self.display = display.into();
        self
    }

    pub fn render(self) -> impl Render {
        let Self {
            label,
            spinner_set,
            tick_interval,
            style,
            spinner_style,
            display,
        } = self;
        let (state, set_state) = signal(ThrobberState::default());
        let spinner_active = Memo::new({
            let display = display.clone();
            move |_| matches!(display.get(), SpinnerDisplay::Spin)
        });

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
                                    .cancel_on_shutdown(&child_token)
                                    .await
                                    .is_err()
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
            })
        };

        on_cleanup(move || {
            cancellation_token.cancel();
        });

        wgt!(state.get(), {
            let mut spinner = Throbber::default();
            if let Some(label) = &label {
                spinner = spinner.label(label.get());
            }
            spinner
                .throbber_set(spinner_set.get())
                .style(style.get())
                .throbber_style(spinner_style.get())
                .use_type(display.get())
        })
    }
}
