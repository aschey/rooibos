use convert_case::{Case, Casing};
use manyhow::{bail, Emitter, ErrorMessage};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::node::{KeyedAttribute, Node, NodeAttribute, NodeElement};
use syn::spanned::Spanned;
use syn::{parse_quote, Block, Expr, ExprLit, Generics, Lit, LitInt};

use crate::{get_import, next_id};

#[derive(Clone, Debug)]
enum Constraint {
    Min,
    Max,
    Percentage,
    Length,
    Ratio,
    Fill,
}

#[derive(Clone)]
enum VarArgTokenStream {
    Single(TokenStream),
    Multi(Vec<TokenStream>),
}

impl VarArgTokenStream {
    fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.is_empty(),
            Self::Multi(s) => s.iter().all(|s| s.is_empty()),
        }
    }
}

impl ToTokens for VarArgTokenStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Single(s) => {
                s.to_tokens(tokens);
            }
            Self::Multi(s) => {
                tokens.append_all(quote!(#(#s),*));
            }
        }
    }
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
        fn_name: Ident,
        props: Option<TokenStream>,
        state: Option<TokenStream>,
    },
    Block {
        fn_name: Ident,
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

        match self.constraint {
            Constraint::Min => quote! { Constraint::Min(#constraint_val) },
            Constraint::Max => quote! { Constraint::Max(#constraint_val) },
            Constraint::Percentage => quote! { Constraint::Percentage(#constraint_val) },
            Constraint::Length => quote! { Constraint::Length(#constraint_val) },
            Constraint::Ratio => quote! { Constraint::Ratio(#constraint_val) },
            Constraint::Fill => quote! { Constraint::Fill(#constraint_val) },
        }
    }

    fn get_overlay_tokens(&self, children: &[View], is_child: bool) -> TokenStream {
        let fn_clones = self.generate_fn_clones();
        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i), true))
            .collect();
        let layout_tokens = quote! {
            move |f: &mut Frame, rect: Rect| {
                #fn_clones
                #(#child_tokens)*
            }
        };

        if is_child {
            quote!((#layout_tokens).view(f, rect);)
        } else {
            layout_tokens
        }
    }

    fn get_layout_tokens(
        &self,
        direction: TokenStream,
        children: &[View],
        child_index: Option<usize>,
        parent_is_overlay: bool,
    ) -> TokenStream {
        let constraints: Vec<_> = children.iter().map(|c| c.get_view_constraint()).collect();

        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i), false))
            .collect();
        let layout_props = self.layout_props.clone();
        let fn_clones = self.generate_fn_clones();

        // If no constraints, auto type inference won't work here
        let constraints_type = if constraints.is_empty() {
            quote!(::<[Constraint; 0]>)
        } else {
            quote!()
        };
        let layout_tokens = quote! {
            move |f: &mut Frame, rect: Rect| {
                #fn_clones
                let layout = Layout::default().direction(#direction);
                let chunks = layout
                    .constraints #constraints_type ([#(#constraints),*])
                    #layout_props
                    .split(rect);
                #(#child_tokens)*
            }
        };

        if let Some(child_index) = child_index {
            if parent_is_overlay {
                quote!((#layout_tokens).view(f, rect);)
            } else {
                quote!((#layout_tokens).view(f, chunks[#child_index]);)
            }
        } else {
            layout_tokens
        }
    }

    fn generate_fn_clones(&self) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children)
            | ViewType::Column(children)
            | ViewType::Overlay(children)
            | ViewType::FocusScope(children) => {
                let child_fns: Vec<_> = children.iter().map(|c| c.generate_fn_clones()).collect();
                quote! { #(#child_fns)* }
            }
            ViewType::Block { fn_name, .. } => {
                quote! {
                    let mut #fn_name = #fn_name.clone();
                }
            }
            ViewType::Element { fn_name, .. } => {
                quote! {
                    let mut #fn_name = #fn_name.clone();
                }
            }
        }
    }

    fn generate_fns(&self) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children)
            | ViewType::Column(children)
            | ViewType::Overlay(children)
            | ViewType::FocusScope(children) => {
                let child_fns: Vec<_> = children.iter().map(|c| c.generate_fns()).collect();
                quote! { #(#child_fns)* }
            }
            ViewType::Block { fn_name, tokens } => {
                quote! {
                    let mut #fn_name = ::std::rc::Rc::new(::std::cell::RefCell::new(
                        move |f: &mut Frame, chunks: Rect| #tokens.view(f, chunks)));
                }
            }
            ViewType::Element {
                name,
                closing_name,
                generics,
                fn_name,
                props,
                state,
            } => {
                let generics = if let Some(generics) = generics {
                    quote!(::#generics)
                } else {
                    quote!()
                };

                let get_container = |inner: TokenStream| {
                    quote! {
                        ::std::rc::Rc::new(::std::cell::RefCell::new(#inner))
                        as ::std::rc::Rc<::std::cell::RefCell<dyn View>>
                    }
                };
                let get_conditional = |rest: TokenStream| {
                    let real = get_container(quote!(#name #rest));

                    // in debug mode, add a dummy condition to associate the closing tag span with
                    // the referenced function so rust analyzer can highlight it
                    // correctly
                    if cfg!(debug_assertions) {
                        let fake = get_container(quote!(#closing_name #rest));
                        quote! {
                            let mut #fn_name = if true {
                                #real
                            } else {
                                #fake
                            };
                        }
                    } else {
                        quote!(let mut #fn_name = #real;)
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

    fn view_to_tokens(&self, child_index: Option<usize>, parent_is_overlay: bool) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children) => self.get_layout_tokens(
                quote! {Direction::Horizontal},
                children,
                child_index,
                parent_is_overlay,
            ),
            ViewType::Column(children) => self.get_layout_tokens(
                quote! {Direction::Vertical},
                children,
                child_index,
                parent_is_overlay,
            ),
            ViewType::FocusScope(children) => children
                .iter()
                .map(|c| c.view_to_tokens(child_index, parent_is_overlay))
                .collect(),
            ViewType::Overlay(children) => self.get_overlay_tokens(children, child_index.is_some()),
            ViewType::Block { fn_name, .. } | ViewType::Element { fn_name, .. } => {
                if let Some(child_index) = child_index {
                    if parent_is_overlay {
                        quote! { (#fn_name).view(f, rect); }
                    } else {
                        quote! { (#fn_name).view(f, chunks[#child_index]); }
                    }
                } else {
                    quote! { (#fn_name) }
                }
            }
        }
    }
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fns = self.generate_fns();
        let view = self.view_to_tokens(None, false);

        tokens.append_all(quote! {
            {
                #fns
                #view
            }
        });
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
        cx_name: Option<&TokenStream>,
        tag: Ident,
        attributes: &[NodeAttribute],
        children: VarArgTokenStream,
        include_parent_id: bool,
        emitter: &mut Emitter,
    ) -> manyhow::Result<Self> {
        Self::from_nodes(
            cx_name,
            Some(tag),
            attributes,
            if children.is_empty() {
                None
            } else {
                Some(children)
            },
            include_parent_id,
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
            "v:fill" => {
                self.constraint = Constraint::Fill;
                self.expr = attribute.value().unwrap().to_token_stream();
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
        cx_name: Option<&TokenStream>,
        tag: Option<Ident>,
        nodes: &[NodeAttribute],
        args: Option<VarArgTokenStream>,
        include_parent_id: bool,
        emitter: &mut Emitter,
    ) -> manyhow::Result<Self> {
        let mut attrs = Self {
            constraint: Constraint::Min,
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
                    if let Some(cx_name) = cx_name {
                        let scope_param = if include_parent_id {
                            let key_param = if let Some(key) = attrs.key.clone() {
                                quote!(Some(#key))
                            } else {
                                quote!(None as Option<String>)
                            };
                            let is_focusable =
                                attrs.focusable.clone().unwrap_or(parse_quote!(false));
                            let caller_id = next_id();
                            quote!(#crate_name::cache::__WIDGET_CACHE.with(|c|
                                c.get_or_create(#cx_name, #is_focusable, #caller_id, #key_param)))
                        } else {
                            cx_name.clone()
                        };
                        attrs.props = Some(quote! { #scope_param, #props.#prop_func });
                    } else {
                        attrs.props = Some(quote! { #props.#prop_func });
                    }
                }
            }
        }

        if let Some(props) = &attrs.props {
            attrs.props = Some(quote! { #props.build() });
        }

        if let Some(tag) = &tag {
            if custom_attrs.is_empty() {
                let props = build_struct(tag, &args);
                if let Some(cx_name) = cx_name {
                    let scope_param = if include_parent_id {
                        let key_param = if let Some(key) = attrs.key.clone() {
                            quote!(Some(#key))
                        } else {
                            quote!(None as Option<String>)
                        };
                        let is_focusable = attrs.focusable.clone().unwrap_or(parse_quote!(false));
                        let caller_id = next_id();
                        quote!(#crate_name::cache::__WIDGET_CACHE.with(|c|
                            c.get_or_create(#cx_name, #is_focusable, #caller_id, #key_param)))
                    } else {
                        cx_name.clone()
                    };
                    attrs.props = Some(quote! { #scope_param, #props.build() });
                } else {
                    attrs.props = Some(quote! { #props.build() });
                }
            }
        }

        Ok(attrs)
    }

    fn from_layout_nodes(nodes: &[NodeAttribute], emitter: &mut Emitter) -> Self {
        let mut attrs = Self {
            constraint: Constraint::Min,
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

fn build_struct(tag_name: &Ident, args: &Option<VarArgTokenStream>) -> TokenStream {
    let caller_id = next_id();
    if let Some(args) = args.as_ref() {
        quote! {
            #tag_name::new(#args).__caller_id(#caller_id)
        }
    } else {
        quote! {
            #tag_name::builder().__caller_id(#caller_id)
        }
    }
}

pub(crate) fn view(
    tokens: TokenStream,
    include_parent_id: bool,
    emitter: &mut Emitter,
) -> manyhow::Result {
    let mut tokens = tokens.into_iter();
    let (cx, comma) = (tokens.next(), tokens.next());
    match (cx, comma) {
        (Some(TokenTree::Ident(cx)), Some(TokenTree::Punct(punct))) if punct.as_char() == ',' => {
            let (nodes, errors) = parse_rstml(tokens.collect());
            let view = parse_root_nodes(&cx.to_token_stream(), nodes, include_parent_id, emitter)?;

            Ok(quote! {
                {
                    #(#errors;)*
                    #view
                }
            })
        }
        _ => {
            bail!("view! macro needs a context and RSX: e.g., view! {{ cx, <row>...</row> }}")
        }
    }
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

        let prop = parse_named_element_children(&nodes, false, emitter)?;
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

fn parse_root_nodes(
    cx_name: &TokenStream,
    nodes: Vec<Node>,
    include_parent_id: bool,
    emitter: &mut Emitter,
) -> manyhow::Result<View> {
    if let [node] = &nodes[..] {
        parse_root_node(cx_name, node, include_parent_id, emitter)
    } else {
        bail!("RSX should contain a single root node")
    }
}

fn parse_root_node(
    cx_name: &TokenStream,
    node: &Node,
    include_parent_id: bool,
    emitter: &mut Emitter,
) -> manyhow::Result<View> {
    if let Node::Element(element) = node {
        parse_element(cx_name, element, include_parent_id, emitter)
    } else {
        bail!(node, "RSX root node should be a named element");
    }
}

fn parse_elements(
    cx_name: &TokenStream,
    nodes: &[Node],
    include_parent_id: bool,
    emitter: &mut Emitter,
) -> manyhow::Result<Vec<View>> {
    let mut views = vec![];
    for node in nodes {
        match node {
            Node::Element(element) => {
                views.push(parse_element(cx_name, element, include_parent_id, emitter)?);
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    let content = get_block_contents(block);
                    views.push(View {
                        view_type: ViewType::Block {
                            tokens: content,
                            fn_name: Ident::new(&format!("__fn{}", next_id()), Span::call_site()),
                        },
                        constraint: Constraint::Min,
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

fn parse_named_element_children(
    nodes: &[Node],
    include_parent_id: bool,
    emitter: &mut Emitter,
) -> manyhow::Result<VarArgTokenStream> {
    let mut tokens = vec![];
    let mut force_vec = false;
    let mut block_count = 0;
    for node in nodes {
        match node {
            Node::Element(element) => {
                let children =
                    parse_named_element_children(&element.children, include_parent_id, emitter)?;
                let attrs = NodeAttributes::from_custom(
                    None,
                    Ident::new(&element.name().to_string(), element.name().span()),
                    element.attributes(),
                    children,
                    include_parent_id,
                    emitter,
                )?;

                if let Some(props) = attrs.props {
                    tokens.push(VarArgTokenStream::Single(quote! { #props }));
                }
            }
            Node::Text(text) => {
                tokens.push(VarArgTokenStream::Single(text.value.to_token_stream()));
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    // Get content without braces
                    let content: TokenStream =
                        block.stmts.iter().map(|s| s.to_token_stream()).collect();
                    tokens.push(VarArgTokenStream::Single(quote! { #content }));

                    block_count += 1;
                }
            }
            Node::Doctype(doctype) => {
                bail!(doctype, "Doctype invalid at this location");
            }
            Node::Fragment(fragment) => {
                let children =
                    parse_named_element_children(&fragment.children, include_parent_id, emitter)?;
                tokens.push(children);
                force_vec = true;
            }
            _ => {}
        }
    }
    Ok(if tokens.is_empty() {
        VarArgTokenStream::Single(TokenStream::default())
    } else if tokens.len() == 1 && !force_vec {
        tokens[0].clone()
    } else if block_count > 1 {
        // Use multiple blocks to pass multiple args to the child
        VarArgTokenStream::Multi(tokens.into_iter().map(|t| quote!(#t)).collect())
    } else {
        VarArgTokenStream::Single(quote! { vec![#(#tokens),*] })
    })
}

fn parse_element(
    cx_name: &TokenStream,
    element: &NodeElement,
    include_parent_id: bool,
    emitter: &mut Emitter,
) -> manyhow::Result<View> {
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
            let children = parse_elements(cx_name, &element.children, include_parent_id, emitter)?;

            Ok(View {
                view_type: ViewType::Row(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        "Col" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(cx_name, &element.children, include_parent_id, emitter)?;

            Ok(View {
                view_type: ViewType::Column(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        "FocusScope" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(cx_name, &element.children, include_parent_id, emitter)?;

            Ok(View {
                view_type: ViewType::FocusScope(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        "Overlay" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes(), emitter);
            let children = parse_elements(cx_name, &element.children, include_parent_id, emitter)?;

            Ok(View {
                view_type: ViewType::Overlay(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                layout_props: attrs.props,
            })
        }
        name => {
            let children =
                parse_named_element_children(&element.children, include_parent_id, emitter)?;

            let attrs = NodeAttributes::from_custom(
                Some(cx_name),
                Ident::new(&(name.to_owned() + "Props"), Span::call_site()),
                element.attributes(),
                children,
                include_parent_id,
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
                    fn_name: Ident::new(&format!("__fn{}", next_id()), Span::call_site()),
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
        lit: Lit::Int(LitInt::new("0", Span::call_site())),
        attrs: vec![],
    })
    .to_token_stream()
}
