use manyhow::manyhow;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Visibility};

#[manyhow]
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, tokens: TokenStream) -> manyhow::Result {
    create_main(attrs, tokens, false, false)
}

#[manyhow]
#[proc_macro_attribute]
pub fn wasm_bindgen(attrs: TokenStream, tokens: TokenStream) -> manyhow::Result {
    create_main(attrs, tokens, true, false)
}

#[manyhow]
#[proc_macro_attribute]
pub fn test(attrs: TokenStream, tokens: TokenStream) -> manyhow::Result {
    create_main(attrs, tokens, false, true)
}

fn create_main(
    attrs: TokenStream,
    tokens: TokenStream,
    is_wasm: bool,
    is_test: bool,
) -> manyhow::Result {
    let mut func: ItemFn = syn::parse2(tokens)?;
    let mut func_copy = func.clone();
    if !is_wasm {
        func.sig.asyncness = None;
    }
    let vis = func_copy.vis.clone();
    func_copy.vis = Visibility::Inherited;
    func_copy.sig.ident = Ident::new(&format!("__{}", func.sig.ident), Span::call_site());

    let reactive = get_reactive_import();
    let output = func.sig.output.clone();

    let func_copy_ident = func_copy.sig.ident.clone();

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

    let test_attr = if is_test { quote!(#[test]) } else { quote!() };
    if is_wasm {
        Ok(quote! {
            #[::wasm_bindgen::prelude::wasm_bindgen(#attrs)]
            #vis #func_sig {
                #reactive::execute_with_owner(move || ___async_main(#func_param_idents)).await
            }

            async fn ___async_main(#inputs) #output {
                #reactive::run_with_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        })
    } else {
        Ok(quote! {
            #test_attr
            #vis #func_sig {
                #reactive::execute_with_owner(move || ___async_main(#func_param_idents))
            }

            #[::tokio::main(#attrs)]
            async fn ___async_main(#inputs) #output {
                #reactive::run_with_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        })
    }
}

fn get_reactive_import() -> TokenStream {
    get_import(
        "rooibos-reactive",
        quote!(reactive),
        quote!(::rooibos_reactive),
    )
}

fn get_import(
    crate_name_str: &str,
    main_self_name: TokenStream,
    direct_self_name: TokenStream,
) -> TokenStream {
    if let Ok(found_crate) = crate_name("rooibos") {
        match found_crate {
            FoundCrate::Itself => quote!(crate::#main_self_name),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident::#main_self_name)
            }
        }
    } else {
        let found_crate = crate_name(crate_name_str).unwrap_or_else(|_| panic!("{crate_name_str} not found"));
        match found_crate {
            FoundCrate::Itself => direct_self_name,
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident)
            }
        }
    }
}
