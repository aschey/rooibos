use manyhow::{bail, manyhow, Emitter, ErrorMessage};
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, ToTokens};
use syn::{parse_str, Attribute, Data, DeriveInput, Expr, Fields};

#[manyhow(proc_macro_derive(Routes, attributes(route)))]
pub fn derive_routes(input: DeriveInput, emitter: &mut Emitter) -> manyhow::Result {
    generate(input, emitter)
}

fn generate(input: DeriveInput, emitter: &mut Emitter) -> manyhow::Result {
    let Data::Enum(enum_input) = input.data else {
        bail!("Router can only be derived on enums")
    };
    let route_attrs: Vec<_> = enum_input
        .variants
        .iter()
        .filter_map(|f| find_route_attr(&f.attrs).map(|a| (f, a)))
        .collect();

    let routes: Vec<_> = route_attrs
        .into_iter()
        .filter_map(|(variant, attr)| {
            let value = match parse_route_attr(&attr) {
                Ok(route_name) => route_name.to_token_stream(),
                Err(e) => {
                    emitter.emit(ErrorMessage::spanned(variant, e));
                    return None;
                }
            };
            let ident = variant.ident.clone();
            let ident: Expr =
                parse_str(&format!("Self::{ident}")).expect("invalid enum expression");
            let fields = match &variant.fields {
                Fields::Named(_) => quote!({ .. }),
                Fields::Unnamed(unnamed) => {
                    let fields = (0..unnamed.unnamed.len()).map(|_| quote!(_));
                    quote!((#(#fields),*))
                }
                Fields::Unit => quote!(),
            };
            Some(quote!(#ident #fields => Some(#value.to_string())))
        })
        .collect();

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let clap_import = get_clap_import()?;

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            fn get_route_from_command(matched: Self) -> Option<String> {
                match matched {
                    #(#routes,)*
                    _ => None,
                }
            }

            fn create_route() -> (#clap_import::ArgMatches, Option<String>) {
                let command = Self::augment_subcommands(#clap_import::Command::default());
                Self::create_route_from(command)
            }

            fn create_route_from(command: #clap_import::Command) -> (#clap_import::ArgMatches, Option<String>) {
                use #clap_import::FromArgMatches as _;

                let matches = command.clone().get_matches();
                let Some((sub, args)) = matches.subcommand() else {
                    return (matches, None);
                };
                let matched = Self::from_arg_matches(&matches).unwrap();
                let Some(mut route) = Self::get_route_from_command(matched) else {
                    return (matches, None);
                };
                if !route.starts_with("/") {
                    route = format!("/{route}");
                }

                let mut path_segments = Vec::new();
                let mut query_params = Vec::new();
                for arg in args.ids() {
                    let mut sub_args = command.find_subcommand(sub).unwrap().get_arguments();
                    let found_arg = sub_args.find(|a| a.get_id() == arg);
                    let Some(found_arg) = found_arg else {
                        continue;
                    };

                    if found_arg.is_required_set() {
                        path_segments.push(
                            args.get_raw(arg.as_str())
                                .unwrap()
                                .next()
                                .unwrap()
                                .to_string_lossy(),
                        );
                    } else {
                        query_params.push(format!(
                            "{}={}",
                            arg.as_str(),
                            args.get_raw(arg.as_str())
                                .unwrap()
                                .next()
                                .unwrap()
                                .to_string_lossy()
                        ));
                    }
                }

                if !path_segments.is_empty() {
                    route += "/";
                    route += &path_segments.join("/");
                }
                if !query_params.is_empty() {
                    route += "?";
                    route += &query_params.join("&");
                }
                (matches, Some(route))
            }
        }
    })
}

fn find_route_attr(attrs: &[Attribute]) -> Option<Attribute> {
    attrs.iter().find(|a| a.path().is_ident("route")).cloned()
}

fn parse_route_attr(attr: &Attribute) -> syn::Result<Ident> {
    let mut ident = Ident::new("_", Span::call_site());
    attr.parse_nested_meta(|meta| {
        let Some(meta_ident) = meta.path.get_ident().cloned() else {
            return Err(meta.error("route attribute should have a single value"))?;
        };
        ident = meta_ident;
        Ok(())
    })?;
    Ok(ident)
}

fn get_clap_import() -> manyhow::Result {
    if let Ok(found_crate) = crate_name("clap") {
        Ok(match found_crate {
            FoundCrate::Itself => quote!(::clap),
            FoundCrate::Name(name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                quote!(::#ident)
            }
        })
    } else {
        bail!("'clap' must be in the crate dependencies to use the Router macro")
    }
}
