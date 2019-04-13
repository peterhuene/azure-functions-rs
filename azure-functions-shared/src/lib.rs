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
mod context;
#[doc(hidden)]
pub mod util;

#[doc(hidden)]
#[allow(renamed_and_removed_lints)]
pub mod rpc {
    use azure_functions_shared_codegen::generated_mod;

    generated_mod!(azure_functions_rpc_messages);

    pub use self::azure_functions_rpc_messages::*;
}

pub use self::context::*;
