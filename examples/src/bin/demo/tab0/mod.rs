use rooibos::prelude::*;

use crate::tab0::charts::{charts, ChartsProps};
use crate::tab0::gauges::{gauges, GaugesProps};

mod charts;
mod gauges;

#[component]
pub(crate) fn Tab0() -> impl Render {
    view! {
        <Col>
            <Gauges constraint=Constraint::Length(9) enhanced_graphics=true/>
            <Charts constraint=Constraint::Min(8) enhanced_graphics=true/>
            <Footer constraint=Constraint::Length(7) />
        </Col>
    }
}

#[component]
fn Footer(constraint: Constraint) -> impl Render {
    view! {
        <Paragraph
            v:constraint=constraint
            block=prop! {
                <Block
                    borders=Borders::ALL
                    title=prop! {
                        <Span magenta bold>
                            "Footer"
                        </Span>
                    }/>
                }
            wrap=prop!(<Wrap trim=true/>)
        >
            <Line>
                "This is a paragraph with several lines.
                You can change style your text the way
                you want"             
            </Line>
            <Line>""</Line>
            <Line>
                <Span>"For example: "</Span>
                <Span red>"under"</Span>
                <Span>" "</Span>
                <Span green>"the"</Span>
                <Span>" "</Span>
                <Span blue>"rainbow"</Span>
                <Span>"."</Span>
            </Line>
            <Line>
                <Span>"Oh and if you didn't "</Span>
                <Span italic>"notice"</Span>
                <Span>" you can "</Span>
                <Span bold>"automatically"</Span>
                <Span>" "</Span>
                <Span reversed>"wrap"</Span>
                <Span>" your "</Span>
                <Span underlined>"text"</Span>
                <Span>"."</Span>
            </Line>
            <Line>
                "One more thing is that it should display unicode characters: 10€"
            </Line>
        </Paragraph>
    }
}
