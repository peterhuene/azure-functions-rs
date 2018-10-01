//! # Azure Functions for Rust
//!
//! This crate supports the code generation for the `azure-functions` crate.
#![feature(rust_2018_preview)]
#![feature(in_band_lifetimes)]
#![feature(proc_macro_diagnostic)]
#![feature(drain_filter)]
#![feature(try_from)]
#![recursion_limit = "128"]
#![deny(unused_extern_crates)]

extern crate azure_functions_shared;
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

mod func;
mod register;
mod util;

use proc_macro::TokenStream;

/// Implements the `register!` macro.
///
/// The `register!` macro is used to register a list of Azure Functions with
/// the Azure Functions host.
///
/// This macro expects a comma-separated list of functions that have the
/// #[func] attribute applied.
///
/// # Examples
///
/// ```rust,ignore
/// azure_functions::register!{
///     module::my_azure_function
/// }
/// ```
#[proc_macro]
pub fn register(input: TokenStream) -> TokenStream {
    register::attr_impl(input)
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
    func::attr_impl(args, input)
}
