use itertools::Itertools;
use manyhow::{Emitter, bail, manyhow};
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Path, Type};

fn is_style(s: &str) -> bool {
    matches!(
        s,
        "Style" | "rooibos_theme::Style" | "ratatui::style::Style"
    )
}

fn is_color(s: &str) -> bool {
    matches!(
        s,
        "Color" | "rooibos_theme::Color" | "ratatui::style::Color"
    )
}

fn filter_fields<F>(fields: &Fields, filter: F) -> Vec<&Ident>
where
    F: Fn(&str) -> bool,
{
    fields
        .iter()
        .filter_map(|f| {
            let ty = f.ty.to_token_stream().to_string().replace(" ", "");
            if filter(ty.as_str()) {
                f.ident.as_ref()
            } else {
                None
            }
        })
        .collect()
}

fn get_style_fields(fields: &Fields) -> Vec<&Ident> {
    filter_fields(fields, is_style)
}

fn get_color_fields(fields: &Fields) -> Vec<&Ident> {
    filter_fields(fields, is_color)
}

fn get_other_fields(fields: &Fields) -> Vec<(&Ident, &Path)> {
    fields
        .iter()
        .filter_map(|f| {
            let ty = f.ty.to_token_stream().to_string().replace(" ", "");
            if !f.attrs.iter().any(|a| a.meta.path().is_ident("subtheme"))
                && !is_color(&ty)
                && !is_style(&ty)
                && let Type::Path(path) = &f.ty
                && path.qself.is_none()
            {
                f.ident.as_ref().map(|i| (i, &path.path))
            } else {
                None
            }
        })
        .collect()
}

#[manyhow(proc_macro_derive(ReactiveTheme, attributes(subtheme, theme)))]
pub fn derive_theme(input: DeriveInput, _emitter: &mut Emitter) -> manyhow::Result {
    let data_struct = if let Data::Struct(data_struct) = &input.data {
        Some(data_struct)
    } else {
        None
    };
    if input.generics.lifetimes().next().is_some() || input.generics.type_params().next().is_some()
    {
        bail!(
            input.span(),
            "Lifetimes and generics are not supported here"
        );
    }
    let struct_name = &input.ident;

    let rooibos_theme = get_import("rooibos-theme");
    let other_fields = data_struct
        .map(|d| get_other_fields(&d.fields))
        .unwrap_or_default();

    let style_trait = Ident::new(
        &(struct_name.to_string() + "ReactiveStyle"),
        Span::call_site(),
    );
    let style_ext_trait = Ident::new(
        &(struct_name.to_string() + "ReactiveStyleExt"),
        Span::call_site(),
    );
    let color_trait = Ident::new(
        &(struct_name.to_string() + "ReactiveColorTheme"),
        Span::call_site(),
    );
    let color_ext_trait = Ident::new(
        &(struct_name.to_string() + "ReactiveColorThemeExt"),
        Span::call_site(),
    );

    let other_traits: TokenStream = other_fields
        .iter()
        .chunk_by(|(_, ty)| ty)
        .into_iter()
        .map(|(ty, group)| {
            let trait_name = Ident::new(
                &(struct_name.to_string()
                    + &ty
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<String>()
                    + "ReactiveExt"),
                Span::call_site(),
            );
            let methods: Vec<_> = group
                .into_iter()
                .map(|(ident, _)| {
                    let base_method = Ident::new(&format!("__internal_{ident}"), Span::call_site());
                    (
                        quote! {
                            fn #ident() -> Signal<#ty>;
                        },
                        quote! {
                            fn #ident() -> Signal<#ty> {
                                let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                                (move || {
                                    trigger.track();
                                    Self::#base_method()
                                })
                                .signal()
                            }
                        },
                    )
                })
                .collect();

            let method_defs: TokenStream = methods.iter().map(|(m, _)| m.clone()).collect();
            let method_impls: TokenStream = methods.into_iter().map(|(_, m)| m).collect();
            quote! {
                pub trait #trait_name {
                    #method_defs
                }

                impl #trait_name for #ty {
                    #method_impls
                }
            }
        })
        .collect();

    let style_fields = data_struct
        .map(|d| get_style_fields(&d.fields))
        .unwrap_or_default();
    let color_fields = data_struct
        .map(|d| get_color_fields(&d.fields))
        .unwrap_or_default();

    let style_trait_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&format!("style_{f}"), Span::call_site());
            quote! {
                fn #style_fn(self) -> Signal<T>;
            }
        })
        .collect();

    let ratatui = get_import("ratatui");

    let style_impl_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&format!("style_{f}"), Span::call_site());
            let base_method = Ident::new(&format!("__internal_style_{f}"), Span::call_site());
            quote! {
                fn #style_fn(self) -> Signal<T> {
                    let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    (move || {
                        trigger.track();
                        Self::#base_method()
                    })
                    .signal()

                }
            }
        })
        .collect();

    let style_signal_impl_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&format!("style_{f}"), Span::call_site());
            quote! {
                fn #style_fn(self) -> Signal<T> {
                    (move || {
                        let this = self.get();
                        this.style_fn().get()
                    })
                    .signal()

                }
            }
        })
        .collect();

    let color_trait_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let fg_fn = Ident::new(&format!("fg_{f}"), Span::call_site());
            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());
            quote! {
                fn #fg_fn(self) -> Signal<T>;
                fn #bg_fn(self) -> Signal<T>;
                fn #underline_fn(self) -> Signal<T>;
            }
        })
        .collect();

    let color_ext_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let color_fn = Ident::new(&f.to_string(), Span::call_site());

            quote! {
                fn #color_fn() -> Signal<Self>;
            }
        })
        .collect();

    let style_ext_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&f.to_string(), Span::call_site());

            quote! {
                fn #style_fn() -> Signal<Self>;
            }
        })
        .collect();

    let impl_fns: TokenStream = data_struct
        .map(|d| {
            d.fields
                .iter()
                .filter_map(|f| {
                    if let Some(ident) = &f.ident {
                        let ty = &f.ty;
                        let base_method =
                            Ident::new(&format!("__internal_{ident}"), Span::call_site());
                        Some(quote! {
                            pub fn #ident() -> Signal<#ty> {
                                let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                                (move || {
                                    trigger.track();
                                    Self::#base_method()
                                }).signal()
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let color_impl_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let fg_fn = Ident::new(&format!("fg_{f}"), Span::call_site());
            let base_fg_fn = Ident::new(&format!("__internal_fg_{f}"), Span::call_site());

            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let base_bg_fn = Ident::new(&format!("__internal_bg_{f}"), Span::call_site());

            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());
            let base_underline_fn =
                Ident::new(&format!("__internal_underline_{f}"), Span::call_site());

            quote! {
                fn #fg_fn(self) -> Signal<T> {
                    let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone());
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        this.clone().#base_fg_fn()
                    }).signal()
                }

                fn #bg_fn(self) -> Signal<T> {
                    let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone());
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        this.clone().#base_bg_fn()
                    }).signal()
                }

                fn #underline_fn(self) -> Signal<T> {
                    let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone());
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        this.clone().#base_underline_fn()
                    }).signal()
                }

            }
        })
        .collect();

    let color_signal_impl_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let fg_fn = Ident::new(&format!("fg_{f}"), Span::call_site());
            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());

            quote! {
                fn #fg_fn(self) -> Signal<T> {
                    let this = self.clone();
                    (move || {
                        this.get().#fg_fn().get()
                    }).signal()
                }

                fn #bg_fn(self) -> Signal<T> {
                    let this = self.clone();
                    (move || {
                        this.get().#bg_fn().get()
                    }).signal()
                }

                fn #underline_fn(self) -> Signal<T> {
                    let this = self.clone();
                    (move || {
                        this.get().#underline_fn().get()
                    }).signal()
                }

            }
        })
        .collect();

    let color_ext_impl_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let color_fn = Ident::new(&f.to_string(), Span::call_site());
            let base_color_fn = Ident::new(&format!("__internal_{f}"), Span::call_site());

            quote! {
                fn #color_fn() -> Signal<Self> {
                    let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone());
                    (move || {
                        trigger.track();
                        Self::#base_color_fn()
                    }).signal()
                }
            }
        })
        .collect();

    let style_ext_impl_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&f.to_string(), Span::call_site());
            let base_style_fn = Ident::new(&format!("__internal_{f}"), Span::call_site());

            quote! {
                fn #style_fn() -> Signal<Self> {
                    let trigger = with_context::<ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    (move || {
                        trigger.track();
                        Self::#base_style_fn()
                    }).signal()
                }
            }
        })
        .collect();
    let signal_marker = Ident::new(&format!("__Signal{struct_name}"), Span::call_site());
    let primitive_marker = Ident::new(&format!("__Primitive{struct_name}"), Span::call_site());
    Ok(quote! {
        pub struct #signal_marker;
        pub struct #primitive_marker;

        impl #struct_name {
            #impl_fns

            pub fn set_global(&self) {
                let context = use_context::<ThemeContext>();
                context.trigger.mark_dirty();
                SetTheme::set_global(self);
            }

            pub fn set_local(&self) {
                let context = use_context::<ThemeContext>();
                context.trigger.mark_dirty();
                SetTheme::set_local(self);
            }
        }

        pub trait #style_trait<T, M> where T: Send + Sync + 'static {
            #style_trait_fns
        }

        impl<T, U> #style_trait<T, #primitive_marker> for U
        where
            U: #rooibos_theme::Styled<Item = T> + Clone + Send + Sync + 'static,
            T: Send + Sync + 'static
        {
            #style_impl_fns
        }

        impl<T, U> #style_trait<T, #signal_marker> for Signal<U>
        where
            U: #style_trait<T, #primitive_marker> + Clone + Send + Sync + 'static,
            T: Clone + Send + Sync + 'static
        {
            #style_signal_impl_fns
        }


        pub trait #color_trait<T, M> where T: Send + Sync + 'static {
            #color_trait_fns
        }

        impl<'a, T, U> #color_trait<T, #primitive_marker> for U
        where
            U: #rooibos_theme::Stylize<'a, T> + Clone + Send + Sync + 'static,
            T: Send + Sync + 'static

        {
            #color_impl_fns
        }

        impl<T, U> #color_trait<T, #signal_marker> for Signal<U>
        where
            U: #color_trait<T, #primitive_marker> + Clone + Send + Sync + 'static,
            T: Clone + Send + Sync + 'static
        {
            #color_signal_impl_fns
        }


        pub trait #color_ext_trait where Self: Send + Sync + Sized + 'static {
            #color_ext_fns
        }

        impl #color_ext_trait for #rooibos_theme::Color {
            #color_ext_impl_fns
        }

        impl #color_ext_trait for #ratatui::style::Color {
            #color_ext_impl_fns
        }

        pub trait #style_ext_trait where Self: Send + Sync + Sized + 'static {
            #style_ext_fns
        }

        impl #style_ext_trait for #rooibos_theme::Style {
            #style_ext_impl_fns
        }

        impl #style_ext_trait for #ratatui::style::Style {
            #style_ext_impl_fns
        }

        #other_traits
    })
}

fn get_import(crate_name_str: &str) -> TokenStream {
    let found_crate = crate_name(crate_name_str).unwrap_or_else(|_| {
        panic!("{crate_name_str} not found. Please add {crate_name_str} to your dependencies.")
    });
    match found_crate {
        FoundCrate::Itself => {
            let ident = Ident::new(&crate_name_str.replace("-", "_"), Span::call_site());
            quote!(#ident)
        }
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    }
}
