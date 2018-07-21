//! # Azure Functions for Rust
//!
//! This crate shares types between the `azure-functions-codegen` and `azure-functions` crates.
#![feature(use_extern_macros)]
#![feature(proc_macro_mod)]
#![feature(proc_macro_gen)]
#![deny(missing_docs)]
#![deny(unused_extern_crates)]

extern crate azure_functions_shared_codegen;
extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[doc(hidden)]
pub mod codegen;
mod context;

#[doc(hidden)]
pub mod rpc {
    pub mod protocol {
        use azure_functions_shared_codegen::generated_mod;

        #[generated_mod]
        mod FunctionRpc {}
        #[generated_mod]
        mod FunctionRpc_grpc {}

        pub use self::FunctionRpc::*;
        pub use self::FunctionRpc_grpc::*;
    }
}

pub use self::context::*;
