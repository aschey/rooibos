use rooibos::reactive::dom::layout::{
    Borders, IntoDimensionSignal, borders, flex_grow, height, min_height,
};
use rooibos::reactive::dom::{Render, span};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::use_context;
use rooibos::reactive::graph::signal::{ReadSignal, RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, wgt};
use rooibos::theme::{Style, Stylize};
use rooibos::tui::symbols;
use rooibos::tui::widgets::{Block, Gauge, LineGauge, Sparkline};

use crate::Tick;
use crate::random::{RandomData, RandomDistribution};

pub(crate) fn gauges(
    enhanced_graphics: bool,
    gauge_height: impl IntoDimensionSignal,
) -> impl Render {
    let (progress, set_progress) = signal(0.0);

    let tick = use_context::<Tick>().unwrap();
    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev
            && seq <= prev
        {
            return seq;
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
        style(
            borders(Borders::all().title("Graphs")),
            height(gauge_height),
            min_height(7)
        ),
        demo_gauge(enhanced_graphics, progress, 2),
        demo_sparkline(enhanced_graphics),
        demo_line_gauge(enhanced_graphics, progress, 2)
    ]
}

fn demo_gauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    gauge_height: impl IntoDimensionSignal,
) -> impl Render {
    wgt!(
        style(height(gauge_height)),
        Gauge::default()
            .block(Block::new().title("Gauge:"))
            .gauge_style(Style::new().magenta().on_black().italic().bold())
            .use_unicode(enhanced_graphics)
            .label(span!("{:.2}%", progress.get() * 100.0))
            .ratio(progress.get())
    )
}

fn demo_line_gauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    gauge_height: impl IntoDimensionSignal,
) -> impl Render {
    wgt!(
        style(height(gauge_height)),
        LineGauge::default()
            .block(Block::new().title("LineGauge:"))
            .filled_style(Style::new().magenta())
            .line_set(if enhanced_graphics {
                symbols::line::THICK
            } else {
                symbols::line::NORMAL
            })
            .ratio(progress.get())
    )
}

fn demo_sparkline(enhanced_graphics: bool) -> impl Render {
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
        if let Some(prev) = prev
            && seq <= prev
        {
            return seq;
        }
        sparkline_signal.update(|s| s.on_tick());
        seq
    });

    wgt!(
        style(flex_grow(1.)),
        Sparkline::default()
            .block(Block::new().title("Sparkline:"))
            .green()
            .data(sparkline_signal.get().points)
            .bar_set(if enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
    )
}
