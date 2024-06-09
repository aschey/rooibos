use manyhow::manyhow;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::parse::Parser;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Visibility};

#[manyhow]
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, tokens: TokenStream) -> manyhow::Result {
    if let Ok(args_parsed) =
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
            .parse2(attrs.clone())
    {
        if !args_parsed.is_empty() && args_parsed.first().unwrap().is_ident("wasm") {
            return create_main(attrs, tokens, true);
        }
    }

    create_main(attrs, tokens, false)
}

fn create_main(attrs: TokenStream, tokens: TokenStream, is_wasm: bool) -> manyhow::Result {
    let mut func: ItemFn = syn::parse2(tokens)?;
    let mut func_copy = func.clone();
    if !is_wasm {
        func.sig.asyncness = None;
    }

    func_copy.vis = Visibility::Inherited;
    func_copy.sig.ident = Ident::new(&format!("__{}", func.sig.ident), Span::call_site());

    let runtime = get_runtime_import();
    let output = func.sig.output.clone();

    let func_copy_ident = func_copy.sig.ident.clone();
    let vis = func_copy.vis.clone();
    let func_sig = func.sig.clone();
    let inputs = func.sig.inputs.clone();
    let func_param_idents: Vec<Box<Pat>> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(PatType { pat, .. }) => Some(pat.clone()),
        })
        .collect();
    let func_param_idents = quote!(#(#func_param_idents),*);
    if is_wasm {
        Ok(quote! {
            #[::wasm_bindgen::prelude::wasm_bindgen(start)]
            #vis #func_sig {
                #runtime::execute(move || ___async_main(#func_param_idents)).await
            }

            async fn ___async_main(#inputs) #output {
                #runtime::init_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        })
    } else {
        Ok(quote! {
            #vis #func_sig {
                #runtime::execute(move || ___async_main(#func_param_idents))
            }

            #[::tokio::main(#attrs)]
            async fn ___async_main(#inputs) #output {
                #runtime::init_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        })
    }
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
