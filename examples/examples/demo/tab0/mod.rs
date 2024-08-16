use rooibos::dom::{col, constraint, line, span, wgt, Render};
use rooibos::tui::layout::Constraint::{self, *};
use rooibos::tui::style::{Modifier, Stylize};
use rooibos::tui::widgets::{Block, Paragraph, Wrap};

use crate::tab0::charts::charts;
use crate::tab0::gauges::gauges;

mod charts;
mod gauges;

pub(crate) fn tab0() -> impl Render {
    col![
        gauges(true, Length(9)),
        charts(true, Min(8)),
        footer(Length(7))
    ]
}

fn footer(footer_constraint: Constraint) -> impl Render {
    wgt![
        props(constraint(footer_constraint)),
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
