use convert_case::{Case, Casing};
use manyhow::{bail, Emitter, ErrorMessage};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::node::{KeyedAttribute, Node, NodeAttribute, NodeElement};
use syn::spanned::Spanned;
use syn::{parse_quote, Block, Expr, ExprLit, Generics, Lit, LitInt};

use crate::get_import;

#[derive(Clone, Debug)]
enum Constraint {
    Min,
    Max,
    Percentage,
    Length,
    Ratio,
}

enum Direction {
    Row,
    Col,
}

#[derive(Clone, Debug)]
enum ViewType {
    Row(Vec<View>),
    Column(Vec<View>),
    Overlay(Vec<View>),
    FocusScope(Vec<View>),
    Element {
        name: Ident,
        closing_name: Ident,
        generics: Option<Generics>,
        props: Option<TokenStream>,
        state: Option<TokenStream>,
    },
    Block {
        tokens: TokenStream,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct View {
    view_type: ViewType,
    constraint: Constraint,
    constraint_val: TokenStream,
    layout_props: Option<TokenStream>,
}

impl View {
    fn get_view_constraint(&self) -> TokenStream {
        let constraint_val = &self.constraint_val;

        let constraint = match self.constraint {
            Constraint::Min => quote! { Constraint::Min(#constraint_val) },
            Constraint::Max => quote! { Constraint::Max(#constraint_val) },
            Constraint::Percentage => quote! { Constraint::Percentage(#constraint_val) },
            Constraint::Length => quote! { Constraint::Length(#constraint_val) },
            Constraint::Ratio => quote! { Constraint::Ratio(#constraint_val) },
        };

        quote!(.constraint(#constraint))
    }

    fn get_overlay_tokens(&self, children: &[View], is_child: bool) -> TokenStream {
        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i), true))
            .collect();
        let layout_tokens = quote! {
            DomNode::overlay()
            #(.child(#child_tokens))*
        };

        layout_tokens
    }

    fn get_layout_tokens(
        &self,
        direction: Direction,
        children: &[View],
        child_index: Option<usize>,
        parent_is_overlay: bool,
    ) -> TokenStream {
        let constraint = self.get_view_constraint();
        let layout = match direction {
            Direction::Row => quote!(row(#constraint)),
            Direction::Col => quote!(col(#constraint)),
        };

        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i), false))
            .collect();
        let layout_props = self.layout_props.clone();

        let layout_tokens = quote! {
            #layout
            #layout_props
            #(.child(#child_tokens))*
        };

        layout_tokens
    }

    // fn generate_fns(&self) -> TokenStream {
    //     match &self.view_type {
    //         ViewType::Row(children)
    //         | ViewType::Column(children)
    //         | ViewType::Overlay(children)
    //         | ViewType::FocusScope(children) => {
    //             let child_fns: Vec<_> = children.iter().map(|c| c.generate_fns()).collect();
    //             quote! { #(#child_fns)* }
    //         }
    //         ViewType::Block { tokens } => {
    //             quote! {
    //                 move |f: &mut Frame, chunks: Rect| #tokens.view(f, chunks)
    //             }
    //         }
    //         ViewType::Element {
    //             name,
    //             closing_name,
    //             generics,
    //             props,
    //             state,
    //         } => {
    //             let generics = if let Some(generics) = generics {
    //                 quote!(::#generics)
    //             } else {
    //                 quote!()
    //             };

    //             let get_conditional = |rest: TokenStream| {
    //                 // in debug mode, add a dummy condition to associate the closing tag span
    // with                 // the referenced function so rust analyzer can highlight it
    //                 // correctly
    //                 if cfg!(debug_assertions) {
    //                     quote! {
    //                         .child(if true {
    //                             #name #rest
    //                         } else {
    //                             #closing_name #rest
    //                         })
    //                     }
    //                 } else {
    //                     quote!(#name #rest)
    //                 }
    //             };
    //             match (props, state) {
    //                 (Some(props), Some(state)) => {
    //                     get_conditional(quote!(#generics (#props, #state)))
    //                 }
    //                 (Some(props), None) => get_conditional(quote!(#generics (#props))),
    //                 (_, _) => get_conditional(quote!(#generics ())),
    //             }
    //         }
    //     }
    // }

    fn view_to_tokens(&self, child_index: Option<usize>, parent_is_overlay: bool) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children) => {
                self.get_layout_tokens(Direction::Row, children, child_index, parent_is_overlay)
            }
            ViewType::Column(children) => {
                self.get_layout_tokens(Direction::Col, children, child_index, parent_is_overlay)
            }
            ViewType::FocusScope(children) => children
                .iter()
                .map(|c| c.view_to_tokens(child_index, parent_is_overlay))
                .collect(),
            ViewType::Overlay(children) => self.get_overlay_tokens(children, child_index.is_some()),
            ViewType::Block { tokens } => tokens.clone(),
            ViewType::Element {
                name,
                closing_name,
                generics,
                props,
                state,
            } => {
                let generics = if let Some(generics) = generics {
                    quote!(::#generics)
                } else {
                    quote!()
                };
                let constraint = self.get_view_constraint();

                let get_conditional = |rest: TokenStream| {
                    // in debug mode, add a dummy condition to associate the closing tag span
                    // the referenced function so rust analyzer can highlight it
                    // correctly
                    if cfg!(debug_assertions) {
                        quote! {
                            if false {
                                #closing_name #rest;
                            }
                            #name #rest
                        }
                    } else {
                        quote!(#name #rest)
                    }
                };

                match (props, state) {
                    (Some(props), Some(state)) => {
                        get_conditional(quote!(#generics (#props, #state)))
                    }
                    (Some(props), None) => get_conditional(quote!(#generics (#props))),
                    (_, _) => get_conditional(quote!(#generics ())),
                }
            }
        }
    }
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let view = self.view_to_tokens(None, false);
        tokens.append_all(view);
    }
}

struct NodeAttributes {
    constraint: Constraint,
    expr: TokenStream,
    props: Option<TokenStream>,
    state: Option<TokenStream>,
    key: Option<Expr>,
    focusable: Option<Expr>,
}

impl NodeAttributes {
    fn from_custom(
        tag: TokenStream,
        attributes: &[NodeAttribute],
        children: TokenStream,
        emitter: &mut Emitter,
    ) -> manyhow::Result<Self> {
        Self::from_nodes(
            Some(tag),
            attributes,
            if children.is_empty() {
                None
            } else {
                Some(children)
            },
            emitter,
        )
    }

    fn parse_standard_attrs(&mut self, attribute: &KeyedAttribute, emitter: &mut Emitter) -> bool {
        match attribute.key.to_string().as_str() {
            "v:min" => {
                self.constraint = Constraint::Min;
                self.expr = attribute.value().unwrap().to_token_stream();
                true
            }
            "v:max" => {
                self.constraint = Constraint::Max;
                self.expr = attribute.value().unwrap().to_token_stream();
                true
            }
            "v:percentage" => {
                self.constraint = Constraint::Percentage;
                self.expr = attribute.value().unwrap().to_token_stream();
                true
            }
            "v:length" => {
                self.constraint = Constraint::Length;
                self.expr = attribute.value().unwrap().to_token_stream();
                true
            }
            "v:ratio" => {
                self.constraint = Constraint::Ratio;
                let val = attribute.value().unwrap().clone();
                let expr = match val {
                    Expr::Tuple(expr) if expr.elems.len() == 2 => expr,
                    _ => {
                        emitter.emit(ErrorMessage::spanned(
                            val,
                            "value should be a tuple of length 2",
                        ));
                        parse_quote!((0, 0))
                    }
                };

                self.expr = expr.elems.to_token_stream();
                true
            }
            "v:state" => {
                if let Some(val) = &attribute.value() {
                    self.state = Some(val.to_token_stream());
                }
                true
            }
            "v:key" => {
                self.key = Some(attribute.value().unwrap().clone());
                true
            }
            "v:focusable" => {
                self.focusable = attribute.value().cloned();
                true
            }
            _ => false,
        }
    }

    fn from_nodes(
        tag: Option<TokenStream>,
        nodes: &[NodeAttribute],
        args: Option<TokenStream>,
        emitter: &mut Emitter,
    ) -> manyhow::Result<Self> {
        let mut attrs = Self {
            constraint: Constraint::Percentage,
            expr: get_default_constraint(),
            props: None,
            state: None,
            key: None,
            focusable: None,
        };

        let custom_attrs: Vec<_> = nodes
            .iter()
            .filter_map(|node| {
                if let NodeAttribute::Attribute(attribute) = node {
                    if !attrs.parse_standard_attrs(attribute, emitter) {
                        return Some(attribute);
                    }
                }
                None
            })
            .collect();

        let crate_name = get_import();

        for attribute in &custom_attrs {
            let func_name = Ident::new(&attribute.key.to_string(), attribute.key.span());
            if let Some(tag) = &tag {
                let prop_func = if let Some(val) = &attribute.value() {
                    quote!(#func_name(#val))
                } else {
                    quote!(#func_name())
                };

                if let Some(props) = attrs.props {
                    attrs.props = Some(quote! {
                        #props.#prop_func
                    });
                } else {
                    let props = build_struct(tag, &args);
                    attrs.props = Some(quote! { #props.#prop_func });
                }
            }
        }

        if let Some(props) = &attrs.props {
            attrs.props = Some(quote! { #props.build() });
        }

        if let Some(tag) = &tag {
            if custom_attrs.is_empty() {
                let props = build_struct(tag, &args);
                attrs.props = Some(quote! { #props.build() });
            }
        }

        Ok(attrs)
    }

    fn from_layout_nodes(nodes: &[NodeAttribute], emitter: &mut Emitter) -> Self {
        let mut attrs = Self {
            constraint: Constraint::Percentage,
            expr: get_default_constraint(),
            props: None,
            state: None,
            key: None,
            focusable: None,
        };

        for node in nodes {
            if let NodeAttribute::Attribute(attribute) = node {
                if !attrs.parse_standard_attrs(attribute, emitter) {
                    let func_name = Ident::new(&attribute.key.to_string(), attribute.key.span());
                    let func = if let Some(val) = &attribute.value() {
                        quote!(#func_name(#val))
                    } else {
                        quote!(#func_name())
                    };

                    if let Some(props) = attrs.props {
                        attrs.props = Some(quote! {
                            #props.#func
                        });
                    } else {
                        attrs.props = Some(quote!(.#func));
                    }
                }
            }
        }

        attrs
    }
}

fn build_struct(tag_name: &TokenStream, args: &Option<TokenStream>) -> TokenStream {
    if let Some(args) = args.as_ref() {
        quote! {
            #tag_name::new(#args)
        }
    } else {
        quote! {
            #tag_name::builder()
        }
    }
}

pub(crate) fn view(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    let (nodes, errors) = parse_rstml(tokens);
    let view = parse_root_nodes(nodes, emitter)?;
    // panic!("{}", quote!(#view));
    Ok(quote! {
        {
            #(#errors;)*
            #view
        }
    })
}

pub(crate) fn prop(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    let (nodes, errors) = parse_rstml(tokens);
    if let [Node::Element(element)] = &nodes[..] {
        let element_name = element.name().to_string();
        if !element_name.is_case(Case::UpperCamel) {
            let element_name_camel = element_name.to_case(Case::UpperCamel);
            bail!(
                element,
                "should have an upper camel case name: {element_name_camel}"
            )
        }

        let prop = parse_named_element_children(&nodes, emitter)?;
        Ok(quote! {
            {
                #(#errors;)*
                #prop
            }
        })
    } else {
        bail!("RSX root node should be a named element");
    }
}

fn parse_rstml(tokens: TokenStream) -> (Vec<Node>, Vec<TokenStream>) {
    let config = rstml::ParserConfig::default().recover_block(true);
    let parser = rstml::Parser::new(config);
    let (nodes, errors) = parser.parse_recoverable(tokens).split_vec();
    let errors: Vec<_> = errors
        .into_iter()
        .map(|e| e.emit_as_expr_tokens())
        .collect();
    (nodes, errors)
}

fn parse_root_nodes(nodes: Vec<Node>, emitter: &mut Emitter) -> manyhow::Result<View> {
    if let [node] = &nodes[..] {
        parse_root_node(node, emitter)
    } else {
        bail!("RSX should contain a single root node")
    }
}

fn parse_root_node(node: &Node, emitter: &mut Emitter) -> manyhow::Result<View> {
    if let Node::Element(element) = node {
        parse_element(element, emitter)
    } else {
        bail!(node, "RSX root node should be a named element");
    }
}

fn parse_elements(nodes: &[Node], emitter: &mut Emitter) -> manyhow::Result<Vec<View>> {
    let mut views = vec![];
    for node in nodes {
        match node {
            Node::Element(element) => {
                views.push(parse_element(element, emitter)?);
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    let content = get_block_contents(block);
                    views.push(View {
                        view_type: ViewType::Block { tokens: content },
                        constraint: Constraint::Percentage,
                        constraint_val: get_default_constraint(),
                        layout_props: None,
                    })
                }
            }
            node => {
                bail!(node, "Invalid RSX node: {node:?}");
            }
        }
    }
    Ok(views)
}

fn parse_named_element_children(nodes: &[Node], emitter: &mut Emitter) -> manyhow::Result {
    let mut tokens = vec![];
    let mut force_vec = false;
    for node in nodes {
        match node {
            Node::Element(element) => {
                let children = parse_named_element_children(&element.children, emitter)?;
                let attrs = NodeAttributes::from_custom(
                    Ident::new(&element.name().to_string(), element.name().span())
                        .to_token_stream(),
                    element.attributes(),
                    children,
                    emitter,
                )?;

                if let Some(props) = attrs.props {
                    tokens.push(quote! { #props });
                }
            }
            Node::Text(text) => {
                tokens.push(text.value.to_token_stream());
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    // Get content without braces
                    let content: TokenStream =
                        block.stmts.iter().map(|s| s.to_token_stream()).collect();

                    tokens.push(quote! { #content });
                }
            }
            Node::Doctype(doctype) => {
                bail!(doctype, "Doctype invalid at this location");
            }
            Node::Fragment(fragment) => {
                let children = parse_named_element_children(&fragment.children, emitter)?;
                tokens.push(children);
                force_vec = true;
            }
            _ => {}
        }
    }
    Ok(if tokens.is_empty() {
        TokenStream::default()
    } else if tokens.len() == 1 && !force_vec {
        tokens[0].clone()
    } else {
        quote! { vec![#(#tokens),*] }
    })
}

fn parse_element(element: &NodeElement, emitter: &mut Emitter) -> manyhow::Result<View> {
    let element_name = element.name().to_string();
    if !element_name.is_case(Case::UpperCamel) {
        let element_name_camel = element_name.to_case(Case::UpperCamel);
        bail!(
            element,
            "should have an upper camel case name: {element_name_camel}"
        )
    }
    match element_name.as_str() {
        "Row" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::Row(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        "Column" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::Column(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        "FocusScope" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::FocusScope(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        "Overlay" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::Overlay(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        name => {
            let children = parse_named_element_children(&element.children, emitter)?;

            let props = Ident::new(&(name.to_owned() + "Props"), Span::call_site());
            let attrs = NodeAttributes::from_custom(
                quote!(move || #props),
                element.attributes(),
                children,
                emitter,
            )?;
            let generics = &element.open_tag.generics;

            Ok(View {
                view_type: ViewType::Element {
                    name: Ident::new(&name.to_case(Case::Snake), element.name().span()),
                    closing_name: Ident::new(
                        &name.to_case(Case::Snake),
                        element
                            .close_tag
                            .clone()
                            .map(|t| t.name.span())
                            .unwrap_or(Span::call_site()),
                    ),
                    generics: if generics.lt_token.is_some() {
                        Some(generics.clone())
                    } else {
                        None
                    },
                    props: attrs.props,
                    state: attrs.state,
                },
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: None,
            })
        }
    }
}

fn get_block_contents(block: &Block) -> TokenStream {
    block.stmts.iter().map(|s| s.to_token_stream()).collect()
}

fn get_default_constraint() -> TokenStream {
    Expr::Lit(ExprLit {
        lit: Lit::Int(LitInt::new("100", Span::call_site())),
        attrs: vec![],
    })
    .to_token_stream()
}
