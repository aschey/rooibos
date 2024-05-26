use rooibos::dom::{col, widget_ref, Constrainable, Render};
use rooibos::tui::layout::Constraint::{self, *};
use rooibos::tui::style::{Modifier, Stylize};
use rooibos::tui::text::{Line, Span};
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

fn footer(constraint: Constraint) -> impl Render {
    widget_ref!(
        Paragraph::new(vec![
            Line::from(
                "This is a paragraph with several lines. You can style your text the way you want"
            ),
            Line::from(""),
            Line::from(vec![
                Span::from("For example: "),
                Span::from("under").red(),
                Span::from(" "),
                Span::from("the").green(),
                Span::from(" "),
                Span::from("rainbow").blue(),
                Span::from(".")
            ]),
            Line::from(vec![
                Span::from("Oh and if you didn't "),
                Span::from("notice").italic(),
                Span::from(" you can "),
                Span::from("automatically").bold(),
                Span::from(" "),
                Span::from("wrap").reversed(),
                Span::from(" your "),
                Span::from("text").underlined(),
                Span::from(".")
            ]),
            Line::from("One more thing is that it should display unicode characters: 10€")
        ])
        .block(
            Block::bordered()
                .title("Footer".magenta())
                .add_modifier(Modifier::BOLD)
        )
        .wrap(Wrap { trim: true })
    )
    .constraint(constraint)
}
