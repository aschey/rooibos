use reactive_graph::owner::StoredValue;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;
use typed_builder::TypedBuilder;

use crate::prelude::*;

#[derive(TypedBuilder, ComponentChildren)]
pub struct Tab {
    header: Line<'static>,
    value: String,
    #[children]
    #[builder(setter(transform = |p: impl IntoChildren| p.into_children()))]
    children: ChildrenFn,
}

#[component]
pub fn Tabs<S>(
    #[prop(children, into)] children: Vec<Tab>,
    #[prop(optional, into)] block: Option<Block<'static>>,
    padding: u16,
    highlight_style: Style,
    current_tab: S,
) -> impl Render
where
    S: Get<Value = String> + Send + Sync + 'static,
{
    let headers = StoredValue::new(
        children
            .iter()
            .map(|t| t.header.clone())
            .collect::<Vec<_>>(),
    );

    let cur_tab = Signal::derive(move || {
        let current_tab = current_tab.get();
        children.iter().enumerate().find_map(|(i, c)| {
            if c.value == current_tab {
                Some((c.children.clone(), i))
            } else {
                None
            }
        })
    });

    let block = StoredValue::new(block);
    view! {
        <Col>
            <TabHeaders
                v:length=padding * 2 + 1
                select=cur_tab.get().unwrap().1
                block_opt=block.get().unwrap()
                highlight_style=highlight_style
            >
                {headers.get().unwrap()}
            </TabHeaders>
            {move || cur_tab.get().unwrap().0()}
        </Col>
    }
}
