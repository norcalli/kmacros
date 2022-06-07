extern crate proc_macro;
use kmacros_shim::*;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, ItemFn};

fn has_attr(attrs: &[syn::Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|a| {
        a.parse_meta()
            .ok()
            .map(|meta| meta.path().is_ident(attr_name))
            .unwrap_or(false)
    })
}

fn has_skip_attr(attrs: &[syn::Attribute]) -> bool {
    has_attr(attrs, "skip")
}

fn has_no_expr_attr(attrs: &[syn::Attribute]) -> bool {
    has_attr(attrs, "no_expr")
}

fn find_ident(pat: &syn::Pat) -> Option<&Ident> {
    match pat {
        syn::Pat::Ident(pat_ident) => Some(&pat_ident.ident),
        _ => None,
    }
}

/// ```
/// use kproc_macros::explain;
///
/// #[explain]
/// fn foo(a: u32, b: f64) -> u32 {
///     let _x = a * b as u32;
///     #[no_expr]
///     let x = a * b as u32;
///     #[skip]
///     let _y = a * b as u32;
///     x * 3
/// }
///
/// struct Foo;
///
/// impl Foo {
///     #[explain]
///     fn bar(&self, a: u32, b: f64) -> u32 {
///         let _x = a * b as u32;
///         #[no_expr]
///         let x = a * b as u32;
///         #[skip]
///         let _y = a * b as u32;
///         x * 3
///     }
/// }
///
/// fn main() {
///     assert_eq!(6, foo(1, 2.));
///     assert_eq!(6, foo_explain(1, 2., |name, expr, value| {
///         println!("{name} {expr:?} {value}");
///     }));
///     assert_eq!(6, Foo.bar(1, 2.));
///     assert_eq!(6, Foo.bar_explain(1, 2., |name, expr, value| {
///         println!("{name} {expr:?} {value}");
///     }));
/// }
/// ```
///
/// Example stdout:
/// ```text
/// STDOUT:
/// ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
/// a None 1
/// b None 2
/// _x Some("a * b as u32") 2
/// x None 2
///  None 6
/// a None 1
/// b None 2
/// _x Some("a * b as u32") 2
/// x None 2
///  None 6
/// ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
/// ```
///
/// See the `tests` for more examples.
#[proc_macro_attribute]
pub fn explain(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let mut new_function = function.clone();

    // TODO wish I could use Span::def_site() but needs nightly
    let callback = Ident::new("callback", Span::call_site());
    let callback_arg: syn::FnArg = parse_quote! {
        mut #callback: impl FnMut(&str, Option<&str>, &dyn std::fmt::Display)
    };

    new_function.sig.inputs.push(callback_arg);

    // TODO wish I could use Span::def_site() but needs nightly
    new_function.sig.ident = Ident::new(
        &format!("{}_explain", function.sig.ident),
        Span::call_site(),
    );

    let new_body = &mut new_function.block;
    new_body.stmts.clear();
    for arg in function.sig.inputs.iter() {
        match arg {
            syn::FnArg::Typed(pattype) if !has_skip_attr(&pattype.attrs) => {
                let ident = or_continue!(find_ident(&pattype.pat));
                let ident_str = ident.to_string();
                let ident_str = ident_str.as_str();
                new_body.stmts.push(parse_quote! {
                    #callback(#ident_str, None, &#ident);
                });
            }
            syn::FnArg::Receiver(_receiver) => (),
            syn::FnArg::Typed(_) => (),
        }
    }
    for stmt in function.block.stmts.iter_mut() {
        match stmt {
            syn::Stmt::Local(local) => {
                let should_skip = has_skip_attr(&local.attrs);
                let skip_expression = has_no_expr_attr(&local.attrs);
                local.attrs.clear();
                new_body.stmts.push(syn::Stmt::Local(local.clone()));
                if should_skip {
                    continue;
                }
                let expr = &or_continue!(local.init.as_ref()).1;
                let ident = or_continue!(find_ident(&local.pat));
                let ident_str = ident.to_string();
                let ident_str = ident_str.as_str();
                let expr_str = expr.to_token_stream().to_string();
                let expr_str = expr_str.as_str();
                let expr_expr: syn::Expr = if skip_expression {
                    parse_quote! { None }
                } else {
                    parse_quote! { Some(#expr_str) }
                };
                new_body.stmts.push(parse_quote! {
                    #callback(#ident_str, #expr_expr, &#ident);
                });
            }
            // syn::Stmt::Item(_item) => (),
            // syn::Stmt::Expr(_expr) => (),
            // syn::Stmt::Semi(_expr, _semi) => (),
            _ => {
                new_body.stmts.push(stmt.clone());
            }
        }
    }

    *new_body = parse_quote! {
        {
            let result = #new_body;
            #callback("", None, &result);
            result
        }
    };

    (quote! {
        #function
        #new_function
    })
    .into()
}

/// ```
/// use kproc_macros::optimized;
///
/// #[optimized]
/// fn foo(a: u32, b: f64) -> u32 {
///     {
///         let mut r = 0.;
///         for _ in 0..a {
///             r += b;
///         }
///         r as u32
///     }
///     {
///         // a * b as u32
///         (a as f64 * b) as u32
///     }
/// }
///
/// fn main() {
///     foo_check(17, 3.1).assert_equal();
///     assert_eq!(foo_slow(17, 3.1), foo(17, 3.1));
/// }
/// ```
/// ```fails
/// use kproc_macros::optimized;
///
/// #[optimized]
/// fn foo(a: u32, b: f64) -> u32 {
///     {
///         let mut r = 0.;
///         for _ in 0..a {
///             r += b;
///         }
///         r as u32
///     }
///     {
///         a * b as u32
///     }
/// }
///
/// fn main() {
///     foo_check(17, 3.1).assert_equal();
///     assert_eq!(foo_slow(17, 3.1), foo(17, 3.1));
/// }
/// ```
/// ```
/// use kproc_macros::optimized;
///
/// #[optimized]
/// fn foo(a: u32, b: f64) -> u32 {
///     {
///         let mut r = 0.;
///         for _ in 0..a {
///             r += b;
///         }
///         r as u32
///     }
///     (a as f64 * b) as u32
/// }
///
/// fn main() {
///     foo_check(17, 3.1).assert_equal();
///     assert_eq!(foo_slow(17, 3.1), foo(17, 3.1));
/// }
/// ```
///
/// See the `tests` for more examples.
#[proc_macro_attribute]
pub fn optimized(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    assert_eq!(
        function.block.stmts.len(),
        2,
        "The optimize function must have two blocks as its body"
    );
    assert!(
        matches!(function.block.stmts[0], syn::Stmt::Expr(_)),
        "The optimize function must have two blocks as its body. \
        First statement isn't an expression."
    );
    assert!(
        matches!(function.block.stmts[1], syn::Stmt::Expr(_)),
        "The optimize function must have two blocks as its body. \
        Second statement isn't an expression."
    );
    let slow_block = &function.block.stmts[0];
    let fast_block = &function.block.stmts[1];
    let slow_body = parse_quote!({
        #slow_block
    });
    let fast_body = parse_quote!({
        #fast_block
    });
    rename! {
        let mut fast_function = function;
    }
    let mut slow_function = fast_function.clone();
    slow_function.block = slow_body;
    fast_function.block = fast_body;

    // TODO wish I could use Span::def_site() but needs nightly
    slow_function.sig.ident = Ident::new(
        &format!("{}_slow", fast_function.sig.ident),
        Span::call_site(),
    );

    let params: Vec<_> = fast_function
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(pattype) => {
                let ident = find_ident(&pattype.pat).expect("Failed to find ident for argument");
                ident.to_token_stream()
            }
            syn::FnArg::Receiver(receiver) => receiver.to_token_stream(),
        })
        .collect();
    let params_types: Vec<_> = fast_function
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(pattype) => {
                pattype.ty.to_token_stream()
            }
            syn::FnArg::Receiver(receiver) => {
                if receiver.reference.is_some() {
                    parse_quote!(&Self)
                } else {
                    parse_quote!(Self)
                }
            },
        })
        .collect();
    let fast_ident = &fast_function.sig.ident;
    let slow_ident = &slow_function.sig.ident;
    let fn_name = fast_ident.to_string();
    let mut check_function = fast_function.clone();
    // TODO wish I could use Span::def_site() but needs nightly
    check_function.sig.ident = Ident::new(
        &format!("{}_check", fast_function.sig.ident),
        Span::call_site(),
    );
    check_function.block = parse_quote! ({
        let fast = #fast_ident(#(#params),*);
        let slow = #slow_ident(#(#params),*);
        kmacros_shim::OptimizeCheckOutput {
            function_name: #fn_name,
            params: (#(#params),*),
            slow,
            fast,
        }
    });
    let fn_ret = match &fast_function.sig.output {
        syn::ReturnType::Default => parse_quote!(()),
        syn::ReturnType::Type(_arrow, ty) => ty.to_token_stream(),
    };
    check_function.sig.output = parse_quote!(-> kmacros_shim::OptimizeCheckOutput<#fn_ret, (#(#params_types),*)>);

    (quote! {
        #fast_function
        #slow_function
        #check_function
    })
    .into()
}
