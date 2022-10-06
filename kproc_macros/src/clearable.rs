//! For structs with named fields generates an impl that would return a doc comment for a field, if
//! one is available:
//!
//!
//! ```rust
//! # use kmacros::FieldIter;
//! #[derive(Debug, FieldIter)]
//! /// outer
//! #[allow(dead_code)]
//! #[field_iter(debug_iter = "dyn Debug")]
//! struct Foo<T> {
//!    x: bool,
//!    b: String,
//!    t: T,
//! }
//!
//!
//! Foo {
//!     x: true,
//!     b: format!("Test"),
//!     t: 64u64,
//! }.debug_iter(|name, value| {
//!   println!("{name} = {value:?}");
//!   None
//! });
//! ```
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    token, AngleBracketedGenericArguments, Attribute, LitStr, Visibility,
};

struct Doc(pub String);
impl Parse for Doc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<token::Eq>()?;
        let s = input.parse::<LitStr>()?.value();
        Ok(Doc(s.trim_start().to_string()))
    }
}

pub struct Top {
    abga: Option<AngleBracketedGenericArguments>,
    ident: Ident,
    where_clause: Option<syn::WhereClause>,
    fields: Vec<Ident>,
    default_fields: Vec<Ident>,
    expressions: Vec<syn::Expr>,
}

impl Parse for Top {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;

        if !input.peek(token::Struct) {
            return Err(input.error("Only structs are supported"));
        }
        input.parse::<token::Struct>()?;
        let ident = input.parse::<Ident>()?;

        let abga = if input.peek(token::Lt) {
            Some(input.parse::<AngleBracketedGenericArguments>()?)
        } else {
            None
        };

        let where_clause = input.parse::<syn::WhereClause>().ok();

        if !input.peek(token::Brace) {
            return Err(input.error("Only structs with named fields are supported"));
        }

        let raw_fields = input.parse::<syn::FieldsNamed>()?;
        let mut default_fields = vec![];
        let mut expressions = vec![];
        let fields = raw_fields
            .named
            .iter()
            .filter_map(|f| {
                let field = f.ident.as_ref()?;
                for attr in &f.attrs {
                    if attr.path.is_ident("clearable") {
                        match attr.parse_meta().expect("invalid clearable attr") {
                            syn::Meta::List(list) => {
                                // In field_iter($(meta),+) now.
                                for meta in list.nested.iter() {
                                    match meta {
                                        syn::NestedMeta::Meta(syn::Meta::Path(p))
                                            if p.is_ident("skip") =>
                                        {
                                            return None;
                                        }
                                        syn::NestedMeta::Meta(syn::Meta::Path(p))
                                            if p.is_ident("default") =>
                                        {
                                            default_fields.push(field.clone());
                                            return None;
                                        }
                                        syn::NestedMeta::Meta(syn::Meta::NameValue(
                                            syn::MetaNameValue {
                                                path,
                                                eq_token: _,
                                                lit: syn::Lit::Str(expr),
                                            },
                                        )) if path.is_ident("expr") => {
                                            let expr = expr
                                                .value()
                                                .replace("{}", &format!("self.{}", field));
                                            let expr: syn::Expr =
                                                syn::parse_str(&expr).expect("Invalid expression");
                                            expressions.push(expr);
                                            return None;
                                        }
                                        syn::NestedMeta::Meta(syn::Meta::NameValue(
                                            syn::MetaNameValue {
                                                path,
                                                eq_token: _,
                                                lit: syn::Lit::Str(expr),
                                            },
                                        )) if path.is_ident("raw_expr") => {
                                            let expr: syn::Expr = syn::parse_str(&expr.value())
                                                .expect("Invalid expression");
                                            expressions.push(expr);
                                            return None;
                                        }
                                        syn::NestedMeta::Meta(meta)
                                            if meta.path().is_ident("skip") =>
                                        {
                                            panic!(
                                                "Invalid skip attr: {}\n\
                                                Only clearable(skip) is valid",
                                                meta.into_token_stream()
                                            )
                                        }
                                        syn::NestedMeta::Meta(meta)
                                            if meta.path().is_ident("default") =>
                                        {
                                            panic!(
                                                "Invalid skip attr: {}\n\
                                                Only clearable(default) is valid",
                                                meta.into_token_stream()
                                            )
                                        }
                                        meta => {
                                            panic!(
                                                "Invalid clearable attr: {}",
                                                meta.into_token_stream()
                                            )
                                        }
                                    }
                                }
                            }
                            meta => {
                                panic!(
                                    "Expected a list of meta attrs for clearable: {}",
                                    meta.into_token_stream()
                                )
                            }
                        }
                    }
                }

                Some(field.clone())
            })
            .collect::<Vec<_>>();

        Ok(Self {
            abga,
            ident,
            where_clause,
            fields,
            default_fields,
            expressions,
        })
    }
}

impl ToTokens for Top {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        let Top {
            abga,
            ident,
            where_clause,
            fields,
            default_fields,
            expressions,
        } = self;

        quote! {
            impl #abga ::kmacros::Clearable for #ident #abga #where_clause {
                fn clear(&mut self) {
                    #(::kmacros::Clearable::clear(&mut self.#fields);)*
                    #(self.#default_fields = Default::default();)*
                    #(#expressions;)*
                }
            }
        }
        .to_tokens(tokens);
    }
}
