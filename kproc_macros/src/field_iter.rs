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
use std::collections::HashSet;
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

/// A function to generate an iterator for
pub struct FunctionJob {
    name: Ident,
    ty: syn::Type,
    fields_to_skip: HashSet<Ident>,
    predicates: Vec<syn::WherePredicate>,
}

pub struct Top {
    abga: Option<AngleBracketedGenericArguments>,
    ident: Ident,
    functions: Vec<FunctionJob>,
    fields: Vec<Ident>,
}

impl Parse for Top {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
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

        let _ = input.parse::<syn::WhereClause>();

        if !input.peek(token::Brace) {
            return Err(input.error("Only structs with named fields are supported"));
        }

        let mut functions: Vec<FunctionJob> = vec![];

        fn parse_bound_meta(functions: &mut Vec<FunctionJob>, m: &syn::MetaList) {
            // field_iter(bound($(bound_meta),*))
            for bound_meta in m.nested.iter() {
                match bound_meta {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path: target_fn,
                        eq_token: _,
                        lit: syn::Lit::Str(predicate),
                    })) => {
                        let target_fn = target_fn.get_ident().expect("Expected an ident for bound");
                        let func =
                            functions
                                .iter_mut()
                                .find(|f| *target_fn == f.name)
                                .expect(&format!(
                                    "Not a generated function: {}",
                                    target_fn.into_token_stream()
                                ));
                        func.predicates
                            .push(predicate.parse().expect("Invalid predicate"));
                    }
                    meta => {
                        panic!("Invalid bound target {}", meta.into_token_stream())
                    }
                }
            }
        }

        fn parse_skip_meta(functions: &mut Vec<FunctionJob>, m: &syn::MetaList, field: &Ident) {
            // field_iter(skip($(fn_skip),*))
            for fn_skip in m.nested.iter() {
                match fn_skip {
                    syn::NestedMeta::Meta(syn::Meta::Path(p)) => {
                        let skip = p.get_ident().expect("Expected an ident for skip");
                        let func = functions
                            .iter_mut()
                            .find(|f| *skip == f.name)
                            .expect(&format!(
                                "Not a generated function: {}",
                                p.into_token_stream()
                            ));
                        func.fields_to_skip.insert(field.clone());
                    }
                    meta => {
                        panic!("Invalid skip target {}", meta.into_token_stream())
                    }
                }
            }
        }

        for attr in attrs.iter() {
            match attr.parse_meta() {
                Ok(syn::Meta::List(list)) if list.path.is_ident("field_iter") => {
                    for job in list.nested.iter() {
                        match job {
                            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                                path,
                                eq_token: _,
                                lit: syn::Lit::Str(ty),
                            })) => {
                                // TODO dups
                                functions.push(FunctionJob {
                                    name: path
                                        .get_ident()
                                        .ok_or_else(|| input.error("Expected ident"))?
                                        .clone(),
                                    ty: ty.parse()?,
                                    fields_to_skip: Default::default(),
                                    predicates: Default::default(),
                                });
                            }
                            syn::NestedMeta::Meta(syn::Meta::List(m))
                                if m.path.is_ident("bound") =>
                            {
                                parse_bound_meta(&mut functions, m);
                            }
                            x => {
                                return Err(input.error(&format!(
                                    "Invalid field_iter attr: {}",
                                    x.into_token_stream()
                                )))
                            }
                        }
                    }
                }
                // TODO invalid field_iter
                _ => (),
            }
        }

        let raw_fields = input.parse::<syn::FieldsNamed>()?;
        let fields = raw_fields
            .named
            .iter()
            .filter_map(|f| {
                let field = f.ident.as_ref()?;
                for attr in &f.attrs {
                    if attr.path.is_ident("field_iter") {
                        match attr.parse_meta().expect("invalid field_iter attr") {
                            syn::Meta::List(list) => {
                                // In field_iter($(meta),+) now.
                                for meta in list.nested.iter() {
                                    match meta {
                                        syn::NestedMeta::Meta(syn::Meta::List(m))
                                            if m.path.is_ident("skip") =>
                                        {
                                            parse_skip_meta(&mut functions, m, field);
                                        }
                                        syn::NestedMeta::Meta(syn::Meta::List(m))
                                            if m.path.is_ident("bound") =>
                                        {
                                            parse_bound_meta(&mut functions, m);
                                        }
                                        syn::NestedMeta::Meta(meta)
                                            if meta.path().is_ident("bound") =>
                                        {
                                            panic!(
                                                "Invalid bound attr: {}\n\
                                                       Expected a bound(fn = \"T: ...\", ...) list",
                                                meta.into_token_stream()
                                            )
                                        }
                                        syn::NestedMeta::Meta(meta)
                                            if meta.path().is_ident("skip") =>
                                        {
                                            panic!(
                                                "Invalid skip attr: {}\n\
                                                       Expected a skip(fn, ...) list",
                                                meta.into_token_stream()
                                            )
                                        }
                                        meta => {
                                            panic!(
                                                "Invalid field_iter attr: {}",
                                                meta.into_token_stream()
                                            )
                                        }
                                    }
                                }
                            }
                            meta => {
                                panic!(
                                    "Expected a list of meta attrs for field_iter: {}",
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
            functions,
            fields,
        })
    }
}

impl ToTokens for Top {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        let Top {
            abga,
            ident,
            functions,
            fields,
        } = self;

        let fns = functions.iter().map(|FunctionJob {
            name,
            ty,
            fields_to_skip,
            predicates,
        }| {
            let is_mut = name.to_string().ends_with("_mut");
            let pred = if predicates.is_empty() {
                vec![]
            } else {
                vec![quote! {
                    where #(#predicates),*
                }]
            };
            if !is_mut {
                let calls = fields.iter().filter_map(|f| {
                    (!fields_to_skip.contains(&f)).then(|| {
                        let fs = f.to_string();
                        quote! {
                            if let Some(x) = f(#fs, &self.#f) {
                                return Some(x);
                            }
                        }
                    })
                });
                quote! {
                    pub fn #name<ZZ>(&self, mut f: impl FnMut(&str, &#ty) -> Option<ZZ>) -> Option<ZZ>
                        #(#pred)*
                    {
                        #(#calls)*
                        None
                    }
                }
            } else {
                let calls = fields.iter().map(|f| {
                    (!fields_to_skip.contains(&f)).then(|| {
                        let fs = f.to_string();
                        quote! {
                            if let Some(x) = f(#fs, &mut self.#f) {
                                return Some(x);
                            }
                        }
                    })
                });
                quote! {
                    pub fn #name<ZZ>(&mut self, mut f: impl FnMut(&str, &mut #ty) -> Option<ZZ>) -> Option<ZZ>
                        #(#pred)*
                    {
                        #(#calls)*
                        None
                    }
                }
            }
        });

        quote! {
            impl #abga #ident #abga {
                #(#fns)*
            }
        }
        .to_tokens(tokens);
    }
}
