use rooibos::prelude::Constraint::*;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::owner::use_context;
use rooibos::reactive::signal::{signal, ReadSignal, RwSignal};
use rooibos::reactive::traits::{Get, Update};

use crate::random::{RandomData, RandomDistribution};
use crate::Tick;

pub(crate) fn gauges(enhanced_graphics: bool, constraint: Constraint) -> impl Render {
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
        demo_gauge(enhanced_graphics, progress, Length(2)),
        demo_sparkline(enhanced_graphics, Length(3)),
        demo_line_gauge(enhanced_graphics, progress, Length(2))
    ]
    .block(Block::bordered().title("Graphs"))
    .constraint(constraint)
}

fn demo_gauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    constraint: Constraint,
) -> impl Render {
    widget_ref!(
        Gauge::default()
            .block(Block::new().title("Gauge:"))
            .gauge_style(Style::new().magenta().on_black().italic().bold())
            .use_unicode(enhanced_graphics)
            .label(format!("{:.2}%", progress.get() * 100.0))
            .ratio(progress.get())
    )
    .constraint(constraint)
}

fn demo_line_gauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    constraint: Constraint,
) -> impl Render {
    widget_ref!(
        LineGauge::default()
            .block(Block::new().title("LineGauge:"))
            .gauge_style(Style::new().magenta())
            .line_set(if enhanced_graphics {
                symbols::line::THICK
            } else {
                symbols::line::NORMAL
            })
            .ratio(progress.get())
    )
    .constraint(constraint)
}

fn demo_sparkline(enhanced_graphics: bool, constraint: Constraint) -> impl Render {
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

    widget_ref!(
        SparklineProps::default()
            .block(Block::new().title("Sparkline:"))
            .green()
            .data(sparkline_signal.get().points)
            .bar_set(if enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
    )
    .constraint(constraint)
}
