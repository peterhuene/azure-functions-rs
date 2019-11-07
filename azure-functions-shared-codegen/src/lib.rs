//! # Azure Functions for Rust
//!
//! This crate supports code generation for the `azure-functions-shared` crate.
#![deny(unused_extern_crates)]
#![recursion_limit = "128"]
#![cfg_attr(feature = "unstable", feature(proc_macro_diagnostic))]
extern crate proc_macro;

mod binding;

use binding::binding_impl;
use proc_macro2::{Delimiter, Span};
use syn::{
    buffer::TokenBuffer, spanned::Spanned, Attribute, AttributeArgs, Ident, Lit, Meta, NestedMeta,
    Path, PathSegment,
};

fn last_segment_in_path(path: &Path) -> &PathSegment {
    path.segments
        .iter()
        .last()
        .expect("expected at least one segment in path")
}

fn parse_attribute_args(attr: &Attribute) -> AttributeArgs {
    let span = attr.span();
    let stream: proc_macro::TokenStream = match TokenBuffer::new2(attr.tokens.clone())
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

fn iter_attribute_args<F>(args: &[NestedMeta], mut callback: F)
where
    F: FnMut(&Ident, &Lit) -> bool,
{
    for arg in args {
        match arg {
            NestedMeta::Meta(m) => {
                match m {
                    Meta::NameValue(nvp) => {
                        if !callback(&last_segment_in_path(&nvp.path).ident, &nvp.lit) {
                            return;
                        }
                    }
                    _ => macro_panic(m.span(), "expected name-value pair for an argument"),
                };
            }
            _ => macro_panic(arg.span(), "expected a name-vaule pair for an argument"),
        };
    }
}

#[cfg(feature = "unstable")]
fn macro_panic<T>(span: Span, message: T) -> !
where
    T: AsRef<str>,
{
    span.unstable().error(message.as_ref()).emit();
    panic!("aborting due to previous error");
}

#[cfg(not(feature = "unstable"))]
fn macro_panic<T>(_: Span, message: T) -> !
where
    T: AsRef<str>,
{
    panic!("{}", message.as_ref());
}

#[proc_macro_attribute]
pub fn binding(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    binding_impl(args, input)
}
