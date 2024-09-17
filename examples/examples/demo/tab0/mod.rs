use rooibos::dom::{line, span};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::layout::{height, pct};
use rooibos::reactive::{col, wgt, Render};
use rooibos::tui::style::{Modifier, Stylize};
use rooibos::tui::widgets::{Block, Paragraph, Wrap};
use taffy::Dimension;

use crate::tab0::charts::charts;
use crate::tab0::gauges::gauges;

mod charts;
mod gauges;

pub(crate) fn tab0() -> impl Render {
    col![
        gauges(true, pct(30.)),
        charts(true, pct(50.)),
        footer(pct(20.))
    ]
}

fn footer(footer_height: Signal<Dimension>) -> impl Render {
    wgt![
        props(height(footer_height)),
        Paragraph::new(vec![
            line!(
                "This is a paragraph with several lines. You can style your text the way you want"
            ),
            line!(""),
            line!(
                span!("For example: "),
                span!("under").red(),
                span!(" "),
                span!("the").green(),
                span!(" "),
                span!("rainbow").blue(),
                span!(".")
            ),
            line!(
                span!("Oh and if you didn't "),
                span!("notice").italic(),
                span!(" you can "),
                span!("automatically").bold(),
                span!(" "),
                span!("wrap").reversed(),
                span!(" your "),
                span!("text").underlined(),
                span!(".")
            ),
            line!("One more thing is that it should display unicode characters: 10€")
        ])
        .block(
            Block::bordered()
                .title("Footer".magenta())
                .add_modifier(Modifier::BOLD)
        )
        .wrap(Wrap { trim: true })
    ]
}
