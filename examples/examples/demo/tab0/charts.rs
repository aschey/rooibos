use ratatui::text::ToSpan;
use rooibos::reactive::dom::layout::{IntoDimensionSignal, full, half, height, show, width};
use rooibos::reactive::dom::widgets::{Chart, Dataset};
use rooibos::reactive::dom::{Render, line, span};
use rooibos::reactive::graph::computed::Memo;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::use_context;
use rooibos::reactive::graph::signal::{ReadSignal, RwSignal};
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{IntoSpan, KeyCode, col, row, wgt};
use rooibos::runtime::use_keypress;
use rooibos::tui::style::{Color, Style, Stylize};
use rooibos::tui::symbols;
use rooibos::tui::widgets::{Axis, BarChart, Block, List, ListItem, ListState};

use crate::Tick;
use crate::random::RandomData;

pub(crate) fn charts(
    enhanced_graphics: bool,
    chart_min_height: impl IntoDimensionSignal,
    show_chart: ReadSignal<bool>,
) -> impl Render {
    row![
        style(height(chart_min_height)),
        col![
            style(width(full()), height(full())),
            row![style(width(full()), height(full())), task_list(), logs()],
            demo_bar_chart(enhanced_graphics)
        ],
        col![
            style(width(full()), height(full()), show(show_chart)),
            demo_chart(enhanced_graphics)
        ]
    ]
}

#[derive(Clone)]
pub struct SinData {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinData {
    pub fn new(interval: f64, period: f64, scale: f64) -> SinData {
        SinData {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinData {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

fn demo_chart(enhanced_graphics: bool) -> impl Render {
    let mut sin1_data = SinData::new(0.2, 3.0, 18.0);
    let sin1 = RwSignal::new(RandomData::<SinData> {
        points: sin1_data.by_ref().take(100).collect(),
        source: sin1_data,
        tick_rate: 5,
    });

    let mut sin2_data = SinData::new(0.1, 2.0, 10.0);
    let sin2 = RwSignal::new(RandomData::<SinData> {
        points: sin2_data.by_ref().take(200).collect(),
        source: sin2_data,
        tick_rate: 10,
    });

    let window = RwSignal::new([0.0, 20.0]);
    let tick = use_context::<Tick>().unwrap();

    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        window.update(|[start, end]| {
            *start += 1.0;
            *end += 1.0;
        });
        sin1.update(|s| s.on_tick());
        sin2.update(|s| s.on_tick());
        seq
    });

    let window_start = Memo::new(move |_| window.get()[0]);
    let window_end = Memo::new(move |_| window.get()[1]);

    wgt!(
        style(height(full())),
        Chart::new(vec![
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .cyan()
                .data(sin1.get().points),
            Dataset::default()
                .name("data3")
                .marker(if enhanced_graphics {
                    symbols::Marker::Braille
                } else {
                    symbols::Marker::Dot
                })
                .yellow()
                .data(sin2.get().points)
        ])
        .block(Block::bordered().title("Chart".cyan().bold()))
        .x_axis(
            Axis::default()
                .title("X Axis")
                .gray()
                .bounds(window.get())
                .labels(vec![
                    window_start.get().into_span(),
                    ((window_start.get() + window_end.get()) / 2.0).into_span(),
                    window_end.get().into_span()
                ])
                .bold()
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .gray()
                .bounds([-20.0, 20.0])
                .labels(vec!["-20".bold(), "  0".to_span(), " 20".bold()])
        )
    )
}

const EVENTS: [(&str, u64); 24] = [
    ("B1", 9),
    ("B2", 12),
    ("B3", 5),
    ("B4", 8),
    ("B5", 2),
    ("B6", 4),
    ("B7", 5),
    ("B8", 9),
    ("B9", 14),
    ("B10", 15),
    ("B11", 1),
    ("B12", 0),
    ("B13", 4),
    ("B14", 6),
    ("B15", 4),
    ("B16", 6),
    ("B17", 4),
    ("B18", 7),
    ("B19", 13),
    ("B20", 8),
    ("B21", 11),
    ("B22", 9),
    ("B23", 3),
    ("B24", 5),
];

fn demo_bar_chart(enhanced_graphics: bool) -> impl Render {
    let bar_chart_data = RwSignal::new(EVENTS.to_vec());

    let tick = use_context::<Tick>().unwrap();

    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        bar_chart_data.update(|data| {
            let event = data.pop().unwrap();
            data.insert(0, event);
        });
        seq
    });

    wgt!(
        style(width(full()), height(full())),
        BarChart::default()
            .block(Block::bordered().title("Bar chart"))
            .data(&bar_chart_data.get())
            .bar_width(3)
            .bar_gap(2)
            .bar_set(if enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
            .value_style(Style::new().black().on_green().italic())
            .label_style(Style::new().yellow())
            .bar_style(Style::new().green())
    )
}

const TASKS: [&str; 24] = [
    "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
    "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
    "Item20", "Item21", "Item22", "Item23", "Item24",
];

fn task_list() -> impl Render {
    let selected_task = RwSignal::<Option<usize>>::new(None);

    let update_current_task = move |change: i32| {
        selected_task.update(|sel| match sel {
            Some(s) => {
                let next = (*s as i32 + change).rem_euclid(TASKS.len() as i32);
                *sel = Some(next as usize);
            }
            None => *sel = Some(0),
        });
    };

    let term_signal = use_keypress();
    Effect::new(move || {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                update_current_task(-1)
            } else if term_signal.code == KeyCode::Down {
                update_current_task(1);
            }
        }
    });

    wgt!(
        style(width(half()), height(full())),
        ListState::default().with_selected(selected_task.get()),
        List::new(TASKS.map(ListItem::new))
            .block(Block::bordered().title("List"))
            .highlight_style(Style::new().bold())
            .highlight_symbol("> ")
    )
}

const LOGS: [(&str, &str); 26] = [
    ("Event1", "INFO"),
    ("Event2", "INFO"),
    ("Event3", "CRITICAL"),
    ("Event4", "ERROR"),
    ("Event5", "INFO"),
    ("Event6", "INFO"),
    ("Event7", "WARNING"),
    ("Event8", "INFO"),
    ("Event9", "INFO"),
    ("Event10", "INFO"),
    ("Event11", "CRITICAL"),
    ("Event12", "INFO"),
    ("Event13", "INFO"),
    ("Event14", "INFO"),
    ("Event15", "INFO"),
    ("Event16", "INFO"),
    ("Event17", "ERROR"),
    ("Event18", "ERROR"),
    ("Event19", "INFO"),
    ("Event20", "INFO"),
    ("Event21", "WARNING"),
    ("Event22", "INFO"),
    ("Event23", "INFO"),
    ("Event24", "WARNING"),
    ("Event25", "INFO"),
    ("Event26", "INFO"),
];

fn logs() -> impl Render {
    let log_data = RwSignal::new(LOGS.to_vec());

    let tick = use_context::<Tick>().unwrap();
    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        log_data.update(|logs| {
            let log = logs.pop().unwrap();
            logs.insert(0, log);
        });
        seq
    });

    let logs = Memo::new(move |_| {
        let info_style = Style::default().fg(Color::Blue);
        let warning_style = Style::default().fg(Color::Yellow);
        let error_style = Style::default().fg(Color::Magenta);
        let critical_style = Style::default().fg(Color::Red);

        log_data
            .get()
            .into_iter()
            .map(|(evt, level)| {
                let style = match level {
                    "ERROR" => error_style,
                    "CRITICAL" => critical_style,
                    "WARNING" => warning_style,
                    _ => info_style,
                };
                (evt, level, style)
            })
            .collect::<Vec<_>>()
    });

    wgt!(
        style(width(half()), height(full())),
        List::new(logs.get().iter().map(|(evt, level, style)| {
            ListItem::new(line!(span!(*style; "{level:<9}"), *evt))
        }))
        .block(Block::bordered().title("Logs"))
    )
}
