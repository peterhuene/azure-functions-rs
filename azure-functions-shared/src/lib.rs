//! # Azure Functions for Rust
//!
//! This crate shares types between the `azure-functions-codegen` and `azure-functions` crates.
#![recursion_limit = "128"]
#![cfg_attr(feature = "unstable", feature(proc_macro_diagnostic))]
#![deny(missing_docs)]
#![deny(unused_extern_crates)]
#![allow(clippy::large_enum_variant)]

#[doc(hidden)]
pub mod codegen;
#[doc(hidden)]
pub mod util;

#[doc(hidden)]
#[allow(clippy::type_repetition_in_bounds)]
pub mod rpc {
    tonic::include_proto!("azure_functions_rpc_messages");
}
