use rooibos::reactive::dom::Render;
use rooibos::reactive::dom::layout::{block, chars, height};
use rooibos::reactive::dom::widgets::Sparkline;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::use_context;
use rooibos::reactive::graph::signal::{ReadSignal, RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{col, wgt};
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::symbols;
use rooibos::tui::widgets::{Block, Gauge, LineGauge};
use taffy::Dimension;

use crate::Tick;
use crate::random::{RandomData, RandomDistribution};

pub(crate) fn gauges(enhanced_graphics: bool, gauge_height: Signal<Dimension>) -> impl Render {
    let (progress, set_progress) = signal(0.0);

    let tick = use_context::<Tick>().unwrap();
    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        set_progress.update(|p| {
            *p = if *p < 1.0 {
                (*p + 0.001f64).min(1.0)
            } else {
                0.0
            }
        });

        seq
    });

    col![
        props(
            block(Block::bordered().title("Graphs")),
            height(gauge_height)
        ),
        demo_gauge(enhanced_graphics, progress, chars(2.)),
        demo_sparkline(enhanced_graphics, chars(3.)),
        demo_line_gauge(enhanced_graphics, progress, chars(2.))
    ]
}

fn demo_gauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    gauge_height: Signal<Dimension>,
) -> impl Render {
    wgt![
        props(height(gauge_height)),
        Gauge::default()
            .block(Block::new().title("Gauge:"))
            .gauge_style(Style::new().magenta().on_black().italic().bold())
            .use_unicode(enhanced_graphics)
            .label(format!("{:.2}%", progress.get() * 100.0))
            .ratio(progress.get())
    ]
}

fn demo_line_gauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    gauge_height: Signal<Dimension>,
) -> impl Render {
    wgt![
        props(height(gauge_height)),
        LineGauge::default()
            .block(Block::new().title("LineGauge:"))
            .filled_style(Style::new().magenta())
            .line_set(if enhanced_graphics {
                symbols::line::THICK
            } else {
                symbols::line::NORMAL
            })
            .ratio(progress.get())
    ]
}

fn demo_sparkline(enhanced_graphics: bool, line_height: Signal<Dimension>) -> impl Render {
    let mut rand_signal = RandomDistribution::new(0, 100);
    let sparkline_points = rand_signal.by_ref().take(300).collect();
    let sparkline_signal = RwSignal::new(RandomData {
        source: rand_signal,
        points: sparkline_points,
        tick_rate: 1,
    });

    let tick = use_context::<Tick>().unwrap();

    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        sparkline_signal.update(|s| s.on_tick());
        seq
    });

    wgt![
        props(height(line_height)),
        Sparkline::default()
            .block(Block::new().title("Sparkline:"))
            .green()
            .data(sparkline_signal.get().points)
            .bar_set(if enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
    ]
}
