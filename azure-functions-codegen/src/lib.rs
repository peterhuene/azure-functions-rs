//! # Azure Functions for Rust
//!
//! This crate supports the code generation for the `azure-functions` crate.
#![recursion_limit = "128"]
#![deny(unused_extern_crates)]
#![cfg_attr(feature = "unstable", feature(proc_macro_diagnostic))]
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;

mod export;
mod func;
mod util;

use proc_macro::TokenStream;

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
pub fn export(input: TokenStream) -> TokenStream {
    export::attr_impl(input)
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
pub fn func(args: TokenStream, input: TokenStream) -> TokenStream {
    func::attr_impl(args, input)
}
