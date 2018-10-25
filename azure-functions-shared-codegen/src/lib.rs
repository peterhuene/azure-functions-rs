//! # Azure Functions for Rust
//!
//! This crate supports code generation for the `azure-functions-shared` crate.

#![feature(rust_2018_preview)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use std::env;
use std::path::Path;
use syn::{parse, ItemMod};

/// A macro to generate a module from code created in the output directory.
///
/// This is a "procedural macro" solution to [https://github.com/rust-lang/rfcs/issues/752](https://github.com/rust-lang/rfcs/issues/752).
///
/// # Examples
///
/// This example uses $OUT_DIR/MyModule.rs to replace the annotated module with
/// the generated code in the output directory:
///
/// ```rust,ignore
/// use azure_functions_shared_codegen::generated_mod;
///
/// #[generated_mod]
/// mod MyModule {}
/// ```
#[proc_macro_attribute]
pub fn generated_mod(_: TokenStream, input: TokenStream) -> TokenStream {
    let m = parse::<ItemMod>(input).unwrap();

    let ident = &m.ident;

    let mut path = Path::new(&env::var("OUT_DIR").unwrap()).join(ident.to_string());

    path.set_extension("rs");

    let path = path.to_str().unwrap().to_string();

    quote!(
        #[path = #path]
        mod #ident;
    )
    .into()
}
