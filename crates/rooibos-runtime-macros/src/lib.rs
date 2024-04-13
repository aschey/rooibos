use manyhow::{bail, manyhow};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Ident, ItemFn};

#[manyhow]
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, tokens: TokenStream) -> manyhow::Result {
    let mut func: ItemFn = syn::parse2(tokens)?;
    if func.sig.inputs.len() != 1 {
        bail!(
            func.sig.inputs,
            "#[rooibos::main] should take a rooibos::runtime::Runtime parameter as the single \
             function argument"
        );
    }
    let original_ident = func.sig.ident.clone();
    func.sig.ident = Ident::new(&format!("__{}", func.sig.ident), func.sig.ident.span());
    let runtime = get_runtime_import();
    let output = func.sig.output.clone();
    let vis = func.vis.clone();
    let ident = func.sig.ident.clone();
    Ok(quote! {
        #vis fn #original_ident() #output {
            #runtime::execute(___async_main)
        }

        #[::tokio::main(#attrs)]
        async fn ___async_main() #output {
            #runtime::init(#ident).await
        }

        #func
    })
}
fn get_runtime_import() -> proc_macro2::TokenStream {
    if let Ok(found_crate) = crate_name("rooibos") {
        match found_crate {
            FoundCrate::Itself => quote::quote!(crate::runtime),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident::runtime)
            }
        }
    } else {
        let found_crate = crate_name("rooibos-dom").expect("rooibos-dom not found");
        match found_crate {
            FoundCrate::Itself => quote::quote!(::rooibos_runtime),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident)
            }
        }
    }
}
