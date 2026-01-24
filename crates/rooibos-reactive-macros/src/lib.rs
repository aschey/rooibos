use manyhow::{Emitter, ErrorMessage, manyhow};
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::Parser;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Visibility, parse_quote};

#[manyhow]
#[proc_macro_attribute]
pub fn main(attrs: TokenStream, tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    create_main(attrs, tokens, false, false, emitter)
}

#[manyhow]
#[proc_macro_attribute]
pub fn wasm_bindgen(
    attrs: TokenStream,
    tokens: TokenStream,
    emitter: &mut Emitter,
) -> manyhow::Result {
    create_main(attrs, tokens, true, false, emitter)
}

#[manyhow]
#[proc_macro_attribute]
pub fn test(attrs: TokenStream, tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    create_main(attrs, tokens, false, true, emitter)
}

fn parse_runtime_flavor_attr(attrs: TokenStream) -> manyhow::Result {
    let mut args_parsed =
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated.parse2(attrs)?;
    let flavor = args_parsed.iter().find(|a| a.path().is_ident("flavor"));
    // Tokio defaults to using the multi-threaded runtime, which makes sense for server-side
    // applications.
    // However, for UIs, a single thread is usually sufficient.
    if flavor.is_none() {
        args_parsed.push(parse_quote!(flavor = "current_thread"));
    }
    Ok(quote!(#args_parsed))
}

fn create_main(
    mut attrs: TokenStream,
    tokens: TokenStream,
    is_wasm: bool,
    is_test: bool,
    emitter: &mut Emitter,
) -> manyhow::Result {
    let mut func: ItemFn = syn::parse2(tokens)?;
    let mut func_copy = func.clone();
    if !is_wasm {
        func.sig.asyncness = None;
        attrs = parse_runtime_flavor_attr(attrs)?;
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
    let res = if is_wasm {
        quote! {
            #[#reactive::__wasm_bindgen::prelude::wasm_bindgen(#attrs)]
            #vis #func_sig {
                #reactive::execute_with_owner_async(___async_main(#func_param_idents)).await
            }

            async fn ___async_main(#inputs) #output {
                #reactive::run_with_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        }
    } else {
        if crate_name("tokio").is_err() {
            emitter.emit(ErrorMessage::call_site(
                "Please add 'tokio' to your dependencies",
            ));
        }
        quote! {
            #test_attr
            #vis #func_sig {
                #reactive::execute_with_owner(move || ___async_main(#func_param_idents))
            }

            #[::#reactive::__tokio::main(#attrs)]
            async fn ___async_main(#inputs) #output {
                #reactive::run_with_executor(#func_copy_ident(#func_param_idents)).await
            }

            #func_copy
        }
    };
    Ok(res)
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
            FoundCrate::Itself => quote!(rooibos::#main_self_name),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident::#main_self_name)
            }
        }
    } else {
        let found_crate =
            crate_name(crate_name_str).unwrap_or_else(|_| panic!("{crate_name_str} not found"));
        match found_crate {
            FoundCrate::Itself => direct_self_name,
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote::quote!(#ident)
            }
        }
    }
}
