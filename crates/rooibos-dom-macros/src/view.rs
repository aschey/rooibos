use convert_case::{Case, Casing};
use manyhow::{bail, Emitter, ErrorMessage};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::node::{KeyedAttribute, Node, NodeAttribute, NodeElement};
use syn::spanned::Spanned;
use syn::{parse_quote, Block, Expr, ExprLit, Generics, Lit, LitInt};

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
    Row {
        span: Span,
        closing_span: Span,
        children: Vec<View>,
    },
    Column {
        span: Span,
        closing_span: Span,
        children: Vec<View>,
    },
    Overlay {
        span: Span,
        closing_span: Span,
        children: Vec<View>,
    },

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
    constraint: Option<ConstraintTokens>,
    id: Option<Expr>,
    layout_props: Option<TokenStream>,
}

impl View {
    fn get_overlay_tokens(&self, span: Span, closing_span: Span, children: &[View]) -> TokenStream {
        let overlay = Ident::new("overlay", span);
        let closing = Ident::new("overlay", closing_span);
        let child_tokens: Vec<_> = children.iter().map(|v| v.view_to_tokens()).collect();
        let constraint = self.constraint.as_ref().map(|c| quote!(.constraint(#c)));
        let id = self.id.as_ref().map(|id| quote!(.id(#id)));
        let overlay_fn = if cfg!(debug_assertions) {
            quote! {
                {
                    if false {
                        #closing()
                    } else {
                        #overlay()
                    }
                }
            }
        } else {
            quote!(#overlay())
        };
        let layout_tokens = quote! {
            #overlay_fn
            #constraint
            #id
            #(.child(#child_tokens))*
        };

        layout_tokens
    }

    fn get_layout_tokens(
        &self,
        direction: Direction,
        span: Span,
        closing_span: Span,
        children: &[View],
    ) -> TokenStream {
        let constraint = self.constraint.as_ref().map(|c| quote!(.constraint(#c)));
        let id = self.id.as_ref().map(|id| quote!(.id(#id)));
        let layout = match direction {
            Direction::Row => Ident::new("row", span),
            Direction::Col => Ident::new("col", span),
        };

        let closing = match direction {
            Direction::Row => Ident::new("row", closing_span),
            Direction::Col => Ident::new("col", closing_span),
        };
        let child_tokens: Vec<_> = children.iter().map(|v| v.view_to_tokens()).collect();
        let layout_props = self.layout_props.clone();

        let layout_fn = if cfg!(debug_assertions) {
            quote! {
                if false {
                    #closing()
                } else {
                    #layout()
                }
            }
        } else {
            quote!(#layout())
        };
        let layout_tokens = quote! {
            #layout_fn
            #constraint
            #id
            #layout_props
            #(.child(#child_tokens))*
        };

        layout_tokens
    }

    fn view_to_tokens(&self) -> TokenStream {
        match &self.view_type {
            ViewType::Row {
                span,
                closing_span,
                children,
            } => self.get_layout_tokens(Direction::Row, *span, *closing_span, children),
            ViewType::Column {
                span,
                closing_span,
                children,
            } => self.get_layout_tokens(Direction::Col, *span, *closing_span, children),
            ViewType::FocusScope(children) => children.iter().map(|c| c.view_to_tokens()).collect(),
            ViewType::Overlay {
                span,
                closing_span,
                children,
            } => self.get_overlay_tokens(*span, *closing_span, children),
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
                let constraint = self.constraint.as_ref().map(|c| quote!(.constraint(#c)));
                let id = self.id.as_ref().map(|id| quote!(.id(#id)));
                let get_conditional = |rest: TokenStream| {
                    // in debug mode, add a dummy condition to associate the closing tag span
                    // the referenced function so rust analyzer can highlight it
                    // correctly
                    if cfg!(debug_assertions) {
                        quote! {
                            {
                                if false {
                                    #closing_name #rest;
                                }
                                #name #rest
                                #constraint
                                #id
                            }
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
        let view = self.view_to_tokens();
        tokens.append_all(view);
    }
}

#[derive(Clone, Debug)]
enum ConstraintTokens {
    Static {
        constraint: Constraint,
        expr: TokenStream,
    },
    Dynamic(TokenStream),
}

impl ToTokens for ConstraintTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            Self::Static { constraint, expr } => match constraint {
                Constraint::Min => quote! { Constraint::Min(#expr) },
                Constraint::Max => quote! { Constraint::Max(#expr) },
                Constraint::Percentage => quote! { Constraint::Percentage(#expr) },
                Constraint::Length => quote! { Constraint::Length(#expr) },
                Constraint::Ratio => quote! { Constraint::Ratio(#expr) },
            },
            Self::Dynamic(expr) => expr.clone(),
        };
        tokens.append_all(new_tokens);
    }
}

struct NodeAttributes {
    constraint: Option<ConstraintTokens>,
    props: Option<TokenStream>,
    state: Option<TokenStream>,
    key: Option<Expr>,
    focusable: Option<Expr>,
    id: Option<Expr>,
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
                self.constraint = Some(ConstraintTokens::Static {
                    constraint: Constraint::Min,
                    expr: attribute.value().unwrap().to_token_stream(),
                });
                true
            }
            "v:max" => {
                self.constraint = Some(ConstraintTokens::Static {
                    constraint: Constraint::Max,
                    expr: attribute.value().unwrap().to_token_stream(),
                });
                true
            }
            "v:percentage" => {
                self.constraint = Some(ConstraintTokens::Static {
                    constraint: Constraint::Percentage,
                    expr: attribute.value().unwrap().to_token_stream(),
                });
                true
            }
            "v:length" => {
                self.constraint = Some(ConstraintTokens::Static {
                    constraint: Constraint::Length,
                    expr: attribute.value().unwrap().to_token_stream(),
                });
                true
            }
            "v:ratio" => {
                let val = attribute.value().unwrap().clone();
                let expr = match val {
                    Expr::Tuple(expr) if expr.elems.len() == 2 => expr.elems,
                    _ => {
                        emitter.emit(ErrorMessage::spanned(
                            val,
                            "value should be a tuple of length 2",
                        ));
                        parse_quote!(0, 0)
                    }
                };

                self.constraint = Some(ConstraintTokens::Static {
                    constraint: Constraint::Ratio,
                    expr: expr.into_token_stream(),
                });
                true
            }
            "v:constraint" => {
                self.constraint = Some(ConstraintTokens::Dynamic(
                    attribute.value().unwrap().into_token_stream(),
                ));
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
            "v:id" => {
                self.id = attribute.value().cloned();
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
            constraint: None,
            props: None,
            state: None,
            key: None,
            focusable: None,
            id: None,
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
            constraint: None,
            props: None,
            state: None,
            key: None,
            focusable: None,
            id: None,
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
            Node::Text(text) => {
                views.push(View {
                    view_type: ViewType::Block {
                        tokens: text.to_token_stream(),
                    },
                    constraint: None,
                    layout_props: None,
                    id: None,
                });
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    let content = get_block_contents(block);
                    views.push(View {
                        view_type: ViewType::Block { tokens: content },
                        constraint: None,
                        layout_props: None,
                        id: None,
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
    let mut block_count = 0;
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
                    block_count += 1;
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
    } else if block_count > 1 {
        quote!(#(#tokens),*)
    } else {
        quote!(vec![#(#tokens),*])
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
                view_type: ViewType::Row {
                    span: element.name().span(),
                    closing_span: element
                        .close_tag
                        .clone()
                        .map(|t| t.name.span())
                        .unwrap_or(Span::call_site()),
                    children,
                },
                constraint: attrs.constraint,
                layout_props: attrs.props,
                id: attrs.id,
            })
        }
        "Col" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::Column {
                    span: element.name().span(),
                    closing_span: element
                        .close_tag
                        .clone()
                        .map(|t| t.name.span())
                        .unwrap_or(Span::call_site()),
                    children,
                },
                constraint: attrs.constraint,
                layout_props: attrs.props,
                id: attrs.id,
            })
        }
        "FocusScope" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::FocusScope(children),
                constraint: attrs.constraint,
                layout_props: attrs.props,
                id: attrs.id,
            })
        }
        "Overlay" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(&element.children, emitter)?;

            Ok(View {
                view_type: ViewType::Overlay {
                    span: element.name().span(),
                    closing_span: element
                        .close_tag
                        .clone()
                        .map(|t| t.name.span())
                        .unwrap_or(Span::call_site()),
                    children,
                },
                constraint: attrs.constraint,
                layout_props: attrs.props,
                id: attrs.id,
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
                layout_props: None,
                id: attrs.id,
            })
        }
    }
}

fn get_block_contents(block: &Block) -> TokenStream {
    block.stmts.iter().map(|s| s.to_token_stream()).collect()
}
