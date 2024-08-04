mod route_derive;

use manyhow::{manyhow, Emitter};
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use syn::DeriveInput;

#[manyhow(proc_macro_derive(Route))]
pub fn derive_route(input: DeriveInput, emitter: &mut Emitter) -> manyhow::Result {
    route_derive::generate(input, emitter)
}

fn get_components_import() -> proc_macro2::TokenStream {
    if let Ok(found_crate) = crate_name("rooibos") {
        match found_crate {
            FoundCrate::Itself => quote::quote!(crate::components),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident::components)
            }
        }
    } else {
        let found_crate = crate_name("rooibos-components").expect("rooibos-components not found");
        match found_crate {
            FoundCrate::Itself => quote::quote!(::rooibos_components),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident)
            }
        }
    }
}
