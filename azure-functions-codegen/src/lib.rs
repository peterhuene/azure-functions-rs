//! # Azure Functions for Rust
//!
//! This crate supports the code generation for the `azure-functions` crate.
#![recursion_limit = "128"]
#![deny(unused_extern_crates)]
#![cfg_attr(feature = "unstable", feature(proc_macro_diagnostic))]
extern crate proc_macro;

mod export;
mod func;

use azure_functions_shared::codegen::macro_panic;
use proc_macro2::{Delimiter, Span};
use syn::{
    buffer::TokenBuffer, spanned::Spanned, token::Eq, Attribute, AttributeArgs, Ident, Lit, LitStr,
    Meta, MetaNameValue, NestedMeta,
};

fn parse_attribute_args(attr: &Attribute) -> AttributeArgs {
    let span = attr.span();
    let stream: proc_macro::TokenStream = match TokenBuffer::new2(attr.tts.clone())
        .begin()
        .group(Delimiter::Parenthesis)
    {
        Some((tree, _, _)) => tree.token_stream().into(),
        None => macro_panic(span, "failed to parse attribute"),
    };

    syn::parse_macro_input::parse::<AttributeArgs>(stream)
        .map_err(move |e| macro_panic(span, format!("failed to parse attribute arguments: {}", e)))
        .unwrap()
}

fn attribute_args_from_name(name: &str, span: Span) -> AttributeArgs {
    vec![NestedMeta::Meta(Meta::NameValue(MetaNameValue {
        ident: Ident::new("name", span),
        eq_token: Eq { spans: [span] },
        lit: Lit::Str(LitStr::new(name, span)),
    }))]
}

/// Implements the `export!` macro.
///
/// The `export!` macro is used to export a list of modules as Azure Functions.
///
/// This macro expects a comma-separated list of module names that implement a
/// function of the same name with the #[func] attribute applied.
///
/// A `FUNCTIONS` constant is declared by the macro.
///
/// # Examples
///
/// ```rust,ignore
/// azure_functions::export! {
///     example
/// }
///
/// fn main() {
///     azure_functions::worker_main(::std::env::args(), FUNCTIONS);
/// }
/// ```
#[proc_macro]
pub fn export(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    export::export_impl(input)
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
/// ```
#[proc_macro_attribute]
pub fn func(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    func::func_impl(args, input)
}
