use rooibos::keybind::key;
use rooibos::reactive::dom::layout::{Borders, Dimension, borders, full, height, val};
use rooibos::reactive::dom::{NodeId, Render, after_render, focus_id, line, text};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::Update;
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{col, wgt};
use rooibos::tui::style::Stylize;

use crate::tab0::charts::charts;
use crate::tab0::gauges::gauges;

mod charts;
mod gauges;

pub(crate) fn tab0() -> impl Render {
    let (show_chart, set_show_chart) = signal(true);
    let id = NodeId::new_auto();
    after_render(move || focus_id(id));

    col![
        props(height(full())),
        gauges(true, "30%"),
        charts(true, "50%", show_chart),
        footer(val("20%"))
    ]
    .focusable(true)
    .id(id)
    .on_key_down(key("t", move |_, _| {
        set_show_chart.update(|s| *s = !*s);
    }))
}

fn footer(footer_height: Signal<Dimension>) -> impl Render {
    wgt![
        props(
            height(footer_height),
            borders(Borders::all().title("Footer".magenta()).bold())
        ),
        text![
            line!(
                "This is a paragraph with several lines. You can style your text the way you want"
            ),
            line!(""),
            line!(
                "For example: ",
                "under".red(),
                " ",
                "the".green(),
                " ",
                "rainbow".blue(),
                "."
            ),
            line!(
                "Oh and if you didn't ",
                "notice".italic(),
                " you can ",
                "automatically".bold(),
                " ",
                "wrap".reversed(),
                " your ",
                "text".underlined(),
                "."
            ),
            line!("One more thing is that it should display unicode characters: 10€")
        ] //        .wrap(Wrap { trim: true })
    ]
}
