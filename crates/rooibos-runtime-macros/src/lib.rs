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

    let runtime = get_runtime_import();
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
                #runtime::execute(move || ___async_main(#func_param_idents)).await
            }

            async fn ___async_main(#inputs) #output {
                #runtime::init_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        })
    } else {
        Ok(quote! {
            #test_attr
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
