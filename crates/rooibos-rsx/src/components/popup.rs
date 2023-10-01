use rooibos_reactive::Scope;

use crate::prelude::*;

#[component]
pub fn Popup<V>(
    cx: Scope,
    #[prop(children)] children: V,
    percent_x: u16,
    percent_y: u16,
) -> impl View
where
    V: LazyView + Clone,
{
    let inverse_y = (100 - percent_y) / 2;
    let inverse_x = (100 - percent_x) / 2;
    move || {
        let mut children = children.clone();
        view! { cx,
            <Column>
                <Row v:percentage=inverse_y />
                <Row v:percentage=percent_y>
                    <Column v:percentage=inverse_x />
                    <Column v:percentage=percent_x>
                        <Overlay>
                            <Clear/>
                            {children}
                        </Overlay>
                    </Column>
                    <Column v:percentage=inverse_x />
                </Row>
                <Row v:percentage=inverse_y />
            </Column>
        }
    }
}
