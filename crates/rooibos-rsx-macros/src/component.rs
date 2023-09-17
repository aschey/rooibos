use attribute_derive::Attribute as AttributeDerive;
use convert_case::{Case, Casing};
use manyhow::{bail, error_message, ErrorMessage};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{
    parse_quote, AngleBracketedGenericArguments, Attribute, FnArg, GenericArgument, Item, ItemFn,
    LitStr, Meta, Pat, PatIdent, Path, PathArguments, ReturnType, Stmt, Type, TypeParamBound,
    TypePath, Visibility,
};

use crate::get_import;

pub struct Model {
    is_transparent: bool,
    docs: Docs,
    vis: Visibility,
    name: Ident,
    scope_name: PatIdent,
    scope_type: Type,
    props: Vec<Prop>,
    body: ItemFn,
    ret: ReturnType,
    view_type: Type,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut item = ItemFn::parse(input)?;
        let docs = Docs::new(&item.attrs)?;

        let props = item
            .sig
            .inputs
            .clone()
            .into_iter()
            .map(Prop::new)
            .collect::<syn::Result<Vec<Prop>>>()?;

        let children_count = props.iter().filter(|p| p.prop_opts.children).count();
        if children_count > 1 {
            bail!(
                item.sig.inputs,
                "only one parameter can be used as children"
            );
        }

        let (scope_name, scope_type) = if props.is_empty() {
            bail!(
                item.sig,
                "this method requires a `Scope` parameter";
                help = "try `fn {}(cx: Scope, /* ... */)`", item.sig.ident
            );
        } else if !is_valid_scope_type(&props[0].ty) {
            bail!(
                item.sig.inputs,
                "this method requires a `Scope` parameter";
                help = "try `fn {}(cx: Scope, /* ... */ */)`", item.sig.ident
            );
        } else {
            (props[0].name.clone(), props[0].ty.clone())
        };

        // We need to remove the `#[doc = ""]` and `#[builder(_)]`
        // attrs from the function signature
        drain_filter(&mut item.attrs, |attr| match &attr.meta {
            Meta::NameValue(attr) => attr.path == parse_quote!(doc),
            Meta::List(attr) => attr.path == parse_quote!(prop),
            _ => false,
        });

        item.sig.inputs.iter_mut().for_each(|arg| {
            if let FnArg::Typed(ty) = arg {
                drain_filter(&mut ty.attrs, |attr| match &attr.meta {
                    Meta::NameValue(attr) => attr.path == parse_quote!(doc),
                    Meta::List(attr) => attr.path == parse_quote!(prop),
                    _ => false,
                });
            }
        });

        let view_type = get_view_generics(&item.sig.output)?;

        Ok(Self {
            is_transparent: false,
            docs,
            vis: item.vis.clone(),
            // create component functions with snake case names to prevent clashes with Ratatui's
            // widget names
            name: convert_to_snake_case(&item.sig.ident, item.sig.ident.span()),
            scope_name,
            scope_type,
            props,
            ret: item.sig.output.clone(),
            body: item,
            view_type,
        })
    }
}

fn is_valid_scope_type(ty: &Type) -> bool {
    [
        parse_quote!(Scope),
        parse_quote!(rooibos::reactive::Scope),
        parse_quote!(::rooibos::reactive::Scope),
        parse_quote!(rooibos_reactive::Scope),
        parse_quote!(::rooibos_reactive::Scope),
    ]
    .iter()
    .any(|test| ty == test)
}

fn get_view_generics(return_type: &ReturnType) -> syn::Result<Type> {
    if let ReturnType::Type(_, return_type) = &return_type {
        if let Type::ImplTrait(impl_trait) = return_type.as_ref() {
            if let Some(TypeParamBound::Trait(bound_trait)) = impl_trait.bounds.first() {
                if let Some(PathArguments::AngleBracketed(args)) =
                    &bound_trait.path.segments.last().map(|s| &s.arguments)
                {
                    if let Some(GenericArgument::Type(generic_type)) = &args.args.first() {
                        return Ok(generic_type.clone());
                    }
                }
            }
        }
    };
    bail!(return_type, "return type is incorrect"; help = "return signature must be `-> impl View<B>`");
}

// implemented manually because Vec::drain_filter is nightly only
// follows std recommended parallel
pub fn drain_filter<T>(vec: &mut Vec<T>, mut some_predicate: impl FnMut(&mut T) -> bool) {
    let mut i = 0;
    while i < vec.len() {
        if some_predicate(&mut vec[i]) {
            _ = vec.remove(i);
        } else {
            i += 1;
        }
    }
}

pub fn convert_to_snake_case(name: &Ident, span: Span) -> Ident {
    let name_str = name.to_string();
    if name_str.is_case(Case::Snake) {
        name.clone()
    } else {
        Ident::new(&name_str.to_case(Case::Snake), span)
    }
}

impl ToTokens for Model {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            is_transparent,
            docs,
            vis,
            name,
            scope_name,
            scope_type,
            props,
            body,
            ret,
            view_type,
        } = self;

        let mut body = body.to_owned();
        let mut props = props.to_owned();

        // check for components that end ;
        if !is_transparent {
            let ends_semi = body.block.stmts.iter().last().and_then(|stmt| match stmt {
                Stmt::Item(Item::Macro(mac)) => mac.semi_token.as_ref(),
                _ => None,
            });
            if let Some(semi) = ends_semi {
                panic!(
                    "{}",
                    error_message!(
                        semi.span(),
                        "A component that ends with a `view!` macro followed by a semicolon will \
                         return (), an empty view. This is usually an accident, not intentional, \
                         so we prevent it. If you'd like to return (), you can do it it \
                         explicitly by returning () as the last item from the component."
                    )
                );
            }
        }

        body.sig.ident = format_ident!(
            "__{}",
            convert_to_snake_case(&body.sig.ident, Span::call_site())
        );
        body.sig.inputs.push(syn::parse_quote!(__parent_id: u64));
        body.sig.output = syn::parse_quote!(-> impl LazyView<#view_type>);
        #[allow(clippy::redundant_clone)] // false positive
        let body_name = body.sig.ident.clone();

        let (impl_generics, generics, where_clause) = body.sig.generics.split_for_impl();
        let generics_tokens: Vec<_> = body
            .sig
            .generics
            .type_params()
            .map(|p| p.ident.to_token_stream())
            .collect();

        if !body.sig.generics.params.is_empty() {
            props.push(Prop {
                docs: Docs::new(&[]).unwrap(),
                prop_opts: PropOpt {
                    default: Some(syn::parse_quote!(Default::default())),
                    ..Default::default()
                },
                name: PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: Ident::new("_phantom", Span::call_site()),
                    subpat: None,
                },
                ty: Type::Path(TypePath {
                    qself: None,
                    path: syn::parse_quote!(::std::marker::PhantomData<(#(#generics_tokens),*)>),
                }),
            });
        }
        let lifetimes = body.sig.generics.lifetimes();

        let props_name = format_ident!("{}Props", name.to_string().to_case(Case::UpperCamel));

        let prop_builder_fields = prop_builder_fields(vis, &props).unwrap();

        let prop_names = prop_names(&props);
        let used_prop_names = prop_names_for_component(&props);
        let builder_name_doc =
            LitStr::new(&format!("Props for the [`{name}`] component."), name.span());

        let component_fn_prop_docs = generate_component_fn_prop_docs(&props).unwrap();

        let crate_import = get_import();

        let mut interior_generics = generics.to_token_stream();
        if !interior_generics.is_empty() {
            interior_generics = quote!(::#interior_generics);
        }

        let component = quote! {
            #crate_import::LazyViewWrapper::new(
                #body_name #interior_generics (#scope_name, #used_prop_names __caller_id))
        };

        let props_arg = quote! {
            props: #props_name #generics
        };

        let destructure_props = quote! {
            let #props_name {
                #prop_names
            } = props;
        };

        let widget_cache_impl = quote! {
            WIDGET_CACHE.with(|c| {
                let mut cache_mut = c.view_cache.borrow_mut();
                if let Some(map) = cache_mut.get_mut::<#crate_import::KeyWrapper<#view_type>>() {
                    if let Some(cache) = map.get_mut(&(__caller_id, #scope_name.id())) {
                        c.mark(cache);
                        cache.stored_view.get_value()
                    } else {
                        let res = ::std::rc::Rc::new(::std::cell::RefCell::new(
                            #component,
                        )) as ::std::rc::Rc<::std::cell::RefCell<
                            dyn #crate_import::View<#view_type>>>;

                        let stored_view =
                        #crate_import::reactive::store_value(#scope_name, res.clone());

                        let mut key_data = #crate_import::KeyData {
                            cx: #scope_name,
                            stored_view,
                            iteration: 0,
                        };
                        c.mark(&mut key_data);
                        map.insert((__caller_id, #scope_name.id()), key_data);
                        res
                    }
                } else {
                    let mut map = ::std::collections::HashMap::<(u64, u64),
                    #crate_import::KeyData<#view_type>>::new();

                    let res = ::std::rc::Rc::new(::std::cell::RefCell::new(#component))
                    as ::std::rc::Rc<::std::cell::RefCell<dyn #crate_import::View<#view_type>>>;

                    let stored_view =
                    #crate_import::reactive::store_value(#scope_name, res.clone());

                    let mut key_data = #crate_import::KeyData {
                        cx: #scope_name,
                        stored_view,
                        iteration: 0,
                    };
                    c.mark(&mut key_data);

                    map.insert((__caller_id, #scope_name.id()), key_data);
                    cache_mut.insert::<#crate_import::KeyWrapper<#view_type>>(map);
                    res
                }
            })
        };

        let output = quote! {
            #[doc = #builder_name_doc]
            #[doc = ""]
            #docs
            #component_fn_prop_docs
            #[caller_id]
            #[derive(#crate_import::typed_builder::TypedBuilder, #crate_import::ComponentChildren)]
            #[builder(doc, crate_module_path=#crate_import::typed_builder)]
            #vis struct #props_name #impl_generics #where_clause {
                #prop_builder_fields
            }

            #docs
            #component_fn_prop_docs
            #[allow(clippy::too_many_arguments, unused_mut)]
            // #tracing_instrument_attr
            #vis fn #name #impl_generics (
                #[allow(unused_variables)]
                #scope_name: #scope_type,
                #props_arg
            ) #ret #(+ #lifetimes)*
            #where_clause
            {
                #[allow(clippy::too_many_arguments, unused_mut)]
                #body
                #destructure_props
                #widget_cache_impl
            }
        };

        tokens.append_all(output)
    }
}

#[derive(Clone, Debug)]
struct Prop {
    docs: Docs,
    prop_opts: PropOpt,
    name: PatIdent,
    ty: Type,
}

impl Prop {
    fn new(arg: FnArg) -> syn::Result<Self> {
        let typed = if let FnArg::Typed(ty) = arg {
            ty
        } else {
            bail!(arg, "receiver not allowed in `fn`");
        };

        let prop_opts = match PropOpt::from_attributes(&typed.attrs) {
            Ok(opts) => opts,
            Err(e) => {
                bail!(e.span(), "{}", e.to_string());
            }
        };

        let name = if let Pat::Ident(i) = *typed.pat {
            i
        } else {
            bail!(
                typed.pat,
                "only `prop: type` style types are allowed within the `#[component]` macro"
            );
        };

        Ok(Self {
            docs: Docs::new(&typed.attrs)?,
            prop_opts,
            name,
            ty: *typed.ty,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Docs(Vec<(String, Span)>);

impl ToTokens for Docs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = self
            .0
            .iter()
            .map(|(doc, span)| quote_spanned!(*span=> #[doc = #doc]))
            .collect::<TokenStream>();

        tokens.append_all(s);
    }
}

impl Docs {
    pub fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        enum ViewCodeFenceState {
            Outside,
            Rust,
            Rsx,
        }
        let mut quotes = "```".to_string();
        let mut quote_ws = "".to_string();
        let mut view_code_fence_state = ViewCodeFenceState::Outside;
        const RUST_START: &str =
            "# ::rooibos::reactive::create_scope(::rooibos::reactive::create_runtime(), |cx| {";
        const RUST_END: &str = "# }).dispose();";
        const RSX_START: &str = "# ::rooibos::rsx::view! {cx,";
        const RSX_END: &str = "# };}).dispose();";

        // Seperated out of chain to allow rustfmt to work
        let map = |(doc, span): (String, Span)| {
            doc.lines()
                .flat_map(|doc| {
                    let trimmed_doc = doc.trim_start();
                    let leading_ws = &doc[..doc.len() - trimmed_doc.len()];
                    let trimmed_doc = trimmed_doc.trim_end();
                    match view_code_fence_state {
                        ViewCodeFenceState::Outside
                            if trimmed_doc.starts_with("```")
                                && trimmed_doc.trim_start_matches('`').starts_with("view") =>
                        {
                            view_code_fence_state = ViewCodeFenceState::Rust;
                            let view = trimmed_doc.find('v').unwrap();
                            quotes = trimmed_doc[..view].to_owned();
                            quote_ws = leading_ws.to_owned();
                            let rust_options = &trimmed_doc[view + "view".len()..].trim_start();
                            vec![
                                format!("{leading_ws}{quotes}{rust_options}"),
                                format!("{leading_ws}{RUST_START}"),
                            ]
                        }
                        ViewCodeFenceState::Rust if trimmed_doc == quotes => {
                            view_code_fence_state = ViewCodeFenceState::Outside;
                            vec![format!("{leading_ws}{RUST_END}"), doc.to_owned()]
                        }
                        ViewCodeFenceState::Rust if trimmed_doc.starts_with('<') => {
                            view_code_fence_state = ViewCodeFenceState::Rsx;
                            vec![format!("{leading_ws}{RSX_START}"), doc.to_owned()]
                        }
                        ViewCodeFenceState::Rsx if trimmed_doc == quotes => {
                            view_code_fence_state = ViewCodeFenceState::Outside;
                            vec![format!("{leading_ws}{RSX_END}"), doc.to_owned()]
                        }
                        _ => vec![doc.to_string()],
                    }
                })
                .map(|l| (l, span))
                .collect::<Vec<_>>()
        };

        let attrs = attrs
            .iter()
            .filter_map(|attr| {
                let Meta::NameValue(attr) = &attr.meta else {
                    return None;
                };
                if !attr.path.is_ident("doc") {
                    return None;
                }

                let Some(val) = value_to_string(&attr.value) else {
                    return Some(Err(error_message!(
                        attr,
                        "expected string literal in value of doc comment"
                    )));
                };

                Some(Ok((val, attr.path.span())))
            })
            .collect::<Result<Vec<_>, ErrorMessage>>()?;

        let mut attrs = attrs.into_iter().flat_map(map).collect::<Vec<_>>();

        if view_code_fence_state != ViewCodeFenceState::Outside {
            if view_code_fence_state == ViewCodeFenceState::Rust {
                attrs.push((format!("{quote_ws}{RUST_END}"), Span::call_site()))
            } else {
                attrs.push((format!("{quote_ws}{RSX_END}"), Span::call_site()))
            }
            attrs.push((format!("{quote_ws}{quotes}"), Span::call_site()))
        }

        Ok(Self(attrs))
    }

    pub fn padded(&self) -> TokenStream {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, (doc, span))| {
                let doc = if idx == 0 {
                    format!("    - {doc}")
                } else {
                    format!("      {doc}")
                };

                let doc = LitStr::new(&doc, *span);

                quote! { #[doc = #doc] }
            })
            .collect()
    }

    pub fn typed_builder(&self) -> String {
        let doc_str = self
            .0
            .iter()
            .map(|s| s.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        if doc_str.chars().filter(|c| *c != '\n').count() != 0 {
            format!("\n\n{doc_str}")
        } else {
            String::new()
        }
    }
}

#[derive(Clone, Debug, AttributeDerive, Default)]
#[attribute(ident = prop)]
struct PropOpt {
    #[attribute(conflicts = [optional_no_strip, strip_option])]
    optional: bool,
    #[attribute(conflicts = [optional, strip_option])]
    optional_no_strip: bool,
    #[attribute(conflicts = [optional, optional_no_strip])]
    strip_option: bool,
    #[attribute(example = "5 * 10")]
    default: Option<syn::Expr>,
    into: bool,
    children: bool,
}

struct TypedBuilderOpts {
    default: bool,
    default_with_value: Option<syn::Expr>,
    strip_option: bool,
    into: bool,
    children: bool,
}

impl TypedBuilderOpts {
    fn from_opts(opts: &PropOpt, is_ty_option: bool) -> Self {
        Self {
            default: opts.optional || opts.optional_no_strip,
            default_with_value: opts.default.clone(),
            strip_option: opts.strip_option || opts.optional && is_ty_option,
            into: opts.into,
            children: opts.children,
        }
    }
}

impl ToTokens for TypedBuilderOpts {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let default = if let Some(v) = &self.default_with_value {
            let v = v.to_token_stream().to_string();
            quote! { default_code=#v, }
        } else if self.default {
            quote! { default, }
        } else {
            quote! {}
        };

        let strip_option = if self.strip_option {
            quote! { strip_option, }
        } else {
            quote! {}
        };

        let into = if self.into {
            quote! { into, }
        } else {
            quote! {}
        };

        let setter = if !strip_option.is_empty() || !into.is_empty() {
            quote! { setter(#strip_option #into) }
        } else {
            quote! {}
        };

        if self.children {
            tokens.append_all(quote! {#[children]});
        }

        if default.is_empty() && setter.is_empty() {
            return;
        }

        let output = quote! { #[builder(#default #setter)] };
        tokens.append_all(output);
    }
}

fn prop_builder_fields(vis: &Visibility, props: &[Prop]) -> syn::Result<TokenStream> {
    let props = props
        .iter()
        .skip(1)
        .map(|prop| {
            let Prop {
                docs,
                name,
                prop_opts,
                ty,
            } = prop;
            let mut name = name.clone();
            name.mutability = None;
            let builder_attrs = TypedBuilderOpts::from_opts(prop_opts, is_option(ty));
            let builder_docs = prop_to_doc(prop, PropDocStyle::Inline)?;

            // Children won't need documentation in many cases
            let allow_missing_docs = if name.ident == "children" {
                quote!(#[allow(missing_docs)])
            } else {
                quote!()
            };

            Ok(quote! {
                #docs
                #builder_docs
                #builder_attrs
                #allow_missing_docs
                #vis #name: #ty,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(props.into_iter().collect())
}

fn prop_names(props: &[Prop]) -> TokenStream {
    let mut props: Vec<_> = props
        .iter()
        .skip(1)
        .map(|Prop { name, .. }| quote! { #name, })
        .collect();
    props.push(quote!(__caller_id));
    props.into_iter().collect()
}

fn prop_names_for_component(props: &[Prop]) -> TokenStream {
    props
        .iter()
        .skip(1)
        .filter(|Prop { name, .. }| {
            let name_str = name.ident.to_string();
            name_str != "_phantom"
        })
        .map(|Prop { name, .. }| {
            let mut name = name.clone();
            name.mutability = None;
            quote! { #name, }
        })
        .collect()
}

fn generate_component_fn_prop_docs(props: &[Prop]) -> syn::Result<TokenStream> {
    let required_prop_docs = props
        .iter()
        .filter(|Prop { prop_opts, .. }| !(prop_opts.optional || prop_opts.optional_no_strip))
        .map(|p| prop_to_doc(p, PropDocStyle::List))
        .collect::<syn::Result<TokenStream>>()?;

    let optional_prop_docs = props
        .iter()
        .filter(|Prop { prop_opts, .. }| prop_opts.optional || prop_opts.optional_no_strip)
        .map(|p| prop_to_doc(p, PropDocStyle::List))
        .collect::<syn::Result<TokenStream>>()?;

    let required_prop_docs = if !required_prop_docs.is_empty() {
        quote! {
            #[doc = "# Required Props"]
            #required_prop_docs
        }
    } else {
        quote! {}
    };

    let optional_prop_docs = if !optional_prop_docs.is_empty() {
        quote! {
            #[doc = "# Optional Props"]
            #optional_prop_docs
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #required_prop_docs
        #optional_prop_docs
    })
}

pub fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let [first] = &segments.iter().collect::<Vec<_>>()[..] {
            first.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

pub fn unwrap_option(ty: &Type) -> syn::Result<Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let [first] = &segments.iter().collect::<Vec<_>>()[..] {
            if first.ident == "Option" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = &first.arguments
                {
                    if let [GenericArgument::Type(ty)] = &args.iter().collect::<Vec<_>>()[..] {
                        return Ok(ty.clone());
                    }
                }
            }
        }
    }

    bail!(
        ty,
        "`Option` must be `std::option::Option`";
        help = "make sure you're not shadowing the `std::option::Option` type \
        that is automatically imported from the standard prelude"
    );
}

#[derive(Clone, Copy)]
enum PropDocStyle {
    List,
    Inline,
}

fn prop_to_doc(
    Prop {
        docs,
        name,
        ty,
        prop_opts,
    }: &Prop,
    style: PropDocStyle,
) -> syn::Result<TokenStream> {
    let ty = if (prop_opts.optional || prop_opts.strip_option) && is_option(ty) {
        unwrap_option(ty)?
    } else {
        ty.to_owned()
    };

    let type_item: syn::Item = parse_quote! {
        type SomeType = #ty;
    };

    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![type_item],
    };

    let pretty_ty = prettyplease::unparse(&file);

    let pretty_ty = &pretty_ty[16..&pretty_ty.len() - 2];

    match style {
        PropDocStyle::List => {
            let arg_ty_doc = LitStr::new(
                &if !prop_opts.into {
                    format!("- **{}**: [`{pretty_ty}`]", quote!(#name))
                } else {
                    format!(
                        "- **{}**: [`impl Into<{pretty_ty}>`]({pretty_ty})",
                        quote!(#name),
                    )
                },
                name.ident.span(),
            );

            let arg_user_docs = docs.padded();

            Ok(quote! {
                #[doc = #arg_ty_doc]
                #arg_user_docs
            })
        }
        PropDocStyle::Inline => {
            let arg_ty_doc = LitStr::new(
                &if !prop_opts.into {
                    format!(
                        "**{}**: [`{}`]{}",
                        quote!(#name),
                        pretty_ty,
                        docs.typed_builder()
                    )
                } else {
                    format!(
                        "**{}**: `impl`[`Into<{}>`]{}",
                        quote!(#name),
                        pretty_ty,
                        docs.typed_builder()
                    )
                },
                name.ident.span(),
            );

            Ok(quote! {
                #[builder(setter(doc = #arg_ty_doc))]
            })
        }
    }
}

fn value_to_string(value: &syn::Expr) -> Option<String> {
    match &value {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Some(s.value()),
            syn::Lit::Char(c) => Some(c.value().to_string()),
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Float(f) => Some(f.base10_digits().to_string()),
            _ => None,
        },
        _ => None,
    }
}
