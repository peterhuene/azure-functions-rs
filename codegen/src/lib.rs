#![feature(proc_macro_diagnostic)]
#![feature(drain_filter)]
#![feature(try_from)]
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate lazy_static;

mod bindings;
mod func;
mod util;

use func::func_attr_impl;
use proc_macro::TokenStream;
use std::convert::TryFrom;
use std::fmt::Write;
use std::path::Path;
use util::PathVec;

/// Implements the `main!` macro.
///
/// The `main!` macro is used to register a list of Azure Functions with
/// the Azure Functions host.
///
/// This macro expects a comma-separated list of functions that have the
/// #[func] attribute applied.
///
/// # Examples
///
/// ```rust,ignore
/// azure_functions::main!{
///     module::my_azure_function
/// }
/// ```
#[proc_macro]
pub fn main(input: TokenStream) -> TokenStream {
    let funcs = match PathVec::try_from(input.clone()) {
        Ok(funcs) => funcs,
        Err(e) => {
            e.emit();
            return input;
        }
    };
    let funcs: Vec<syn::Expr> = funcs
        .into_iter()
        .map(|path| {
            let mut expr = String::new();
            if path.leading_colon.is_some() {
                expr += "::";
            }

            let mut segments = path.segments.into_iter().peekable();
            while let Some(segment) = segments.next() {
                if segments.peek().is_some() {
                    write!(&mut expr, "{}::", segment.ident).unwrap();
                } else {
                    write!(
                        &mut expr,
                        "__{}_FUNCTION",
                        segment.ident.to_string().to_uppercase()
                    ).unwrap();
                }
            }

            syn::parse_str::<syn::Expr>(&expr).unwrap()
        })
        .collect();

    let expanded = quote!{
        pub fn main() {
            azure_functions::worker_main(
                std::env::args(),
                &[#(&#funcs),*],
            );
        }
    };

    expanded.into()
}

/// Implements the `func` attribute.
///
/// This attribute is used to turn a Rust function into an Azure Function.
///
/// # Examples
///
/// ```rust,ignore
/// use azure_functions::func;
/// use azure_functions::bindings::HttpRequest;
///
/// #[func]
/// pub fn example(req: &HttpRequest) {
/// }
#[proc_macro_attribute]
pub fn func(args: TokenStream, input: TokenStream) -> TokenStream {
    func_attr_impl(args, input)
}

/// This exists to enable us to create a module from generated code.
/// A proc_macro solution to https://github.com/rust-lang/rfcs/issues/752
#[doc(hidden)]
#[proc_macro_attribute]
pub fn generated_mod(_: TokenStream, input: TokenStream) -> TokenStream {
    let m = syn::parse::<syn::ItemMod>(input.clone()).unwrap();

    let ident = &m.ident;

    let mut path = Path::new(&std::env::var("OUT_DIR").unwrap()).join(ident.to_string());

    path.set_extension("rs");

    let path = path.to_str().unwrap().to_string();

    quote!(
        #[path = #path]
        mod #ident;
    ).into()
}
