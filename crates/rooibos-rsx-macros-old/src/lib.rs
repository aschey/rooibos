use std::sync::atomic::{AtomicU64, Ordering};

use manyhow::{manyhow, Emitter};
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::ToTokens;
use syn::DeriveInput;

use crate::view_traits::attr_make_builder;

mod caller_id;
mod component;
mod component_children;
mod view;
mod view_traits;
mod widget;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

#[manyhow]
#[proc_macro]
pub fn prop(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    view::prop(tokens, emitter)
}

#[manyhow]
#[proc_macro]
pub fn view(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    view::view(tokens, true, emitter)
}

#[manyhow]
#[proc_macro]
pub fn mount(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    view::view(tokens, false, emitter)
}

#[manyhow]
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, tokens: TokenStream) -> manyhow::Result {
    let model: component::Model = syn::parse2(tokens)?;
    Ok(model.into_token_stream())
}

#[manyhow]
#[proc_macro_attribute]
pub fn caller_id(_attr: TokenStream, tokens: TokenStream) -> manyhow::Result {
    let input: DeriveInput = syn::parse2(tokens)?;
    caller_id::parse(input)
}

#[manyhow]
#[proc_macro_derive(ComponentChildren, attributes(children))]
pub fn component_children(tokens: TokenStream) -> manyhow::Result {
    let input: DeriveInput = syn::parse2(tokens)?;
    component_children::parse(input)
}

#[manyhow]
#[proc_macro]
pub fn impl_widget(tokens: TokenStream) -> manyhow::Result {
    let input: widget::Model = syn::parse2(tokens)?;
    Ok(input.into_token_stream())
}

#[manyhow]
#[proc_macro]
pub fn impl_stateful_widget(tokens: TokenStream) -> manyhow::Result {
    let mut input: widget::Model = syn::parse2(tokens)?;
    input.stateful = true;
    Ok(input.into_token_stream())
}

#[manyhow]
#[proc_macro_derive(Widget, attributes(make_builder_trait))]
pub fn widget(tokens: TokenStream) -> manyhow::Result {
    let input: DeriveInput = syn::parse2(tokens)?;
    Ok(widget::derive_widget(input))
}

#[manyhow]
#[proc_macro_derive(StatefulWidget)]
pub fn stateful_widget(tokens: TokenStream) -> manyhow::Result {
    let input: DeriveInput = syn::parse2(tokens)?;
    Ok(widget::derive_stateful_widget(input))
}

#[manyhow]
#[proc_macro_attribute]
pub fn make_builder(attr: TokenStream, tokens: TokenStream) -> manyhow::Result {
    Ok(attr_make_builder(attr, tokens))
}

#[manyhow]
#[proc_macro]
pub fn impl_stateful_render(tokens: TokenStream) -> manyhow::Result {
    let input: view_traits::StatefulModel = syn::parse2(tokens)?;
    Ok(input.into_token_stream())
}

fn get_import() -> proc_macro2::TokenStream {
    if let Ok(found_crate) = crate_name("rooibos-old") {
        match found_crate {
            FoundCrate::Itself => quote::quote!(crate::rsx),
            FoundCrate::Name(name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                quote::quote!(#ident::rsx)
            }
        }
    } else {
        let found_crate = crate_name("rooibos-rsx-old").expect("rooibos-rsx-old not found");
        match found_crate {
            FoundCrate::Itself => quote::quote!(crate),
            FoundCrate::Name(name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                quote::quote!(#ident)
            }
        }
    }
}
