use rooibos::prelude::Constraint::*;
use rooibos::prelude::*;

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
            Line::new(
                "This is a paragraph with several lines. You can style your text the way you want"
            ),
            Line::new(""),
            Line::new(vec![
                Span::new("For example: "),
                Span::new("under").red(),
                Span::new(" "),
                Span::new("the").green(),
                Span::new(" "),
                Span::new("rainbow").blue(),
                Span::new(".")
            ]),
            Line::new(vec![
                Span::new("Oh and if you didn't "),
                Span::new("notice").italic(),
                Span::new(" you can "),
                Span::new("automatically").bold(),
                Span::new(" "),
                Span::new("wrap").reversed(),
                Span::new(" your "),
                Span::new("text").underlined(),
                Span::new(".")
            ]),
            Line::new("One more thing is that it should display unicode characters: 10€")
        ])
        .block(
            Block::bordered()
                .title("Footer".magenta())
                .add_modifier(Modifier::BOLD)
        )
    )
    .constraint(constraint)
}
