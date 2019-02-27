//! # Azure Functions for Rust
//!
//! This crate supports code generation for the `azure-functions-shared` crate.
#![deny(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::env;
use std::path::Path;
use syn::{parse, Ident};

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
#[proc_macro]
pub fn generated_mod(input: TokenStream) -> TokenStream {
    let ident = parse::<Ident>(input).unwrap();

    let mut path = Path::new(&env::var("OUT_DIR").unwrap()).join(ident.to_string());

    path.set_extension("rs");

    let path = path.to_str().unwrap().to_string();

    quote!(
        #[path = #path]
        mod #ident;
    )
    .into()
}
