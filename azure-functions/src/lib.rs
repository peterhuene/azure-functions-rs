//! # Azure Functions for Rust
//!
//! The Azure Functions for Rust crate supports writting Azure Functions in Rust.
//!
//! The following Azure Functions trigger bindings are supported:
//!
//! * [HTTP triggers](bindings/struct.HttpRequest.html)
//! * [Timer triggers](bindings/struct.TimerInfo.html)
//! * [Queue triggers](bindings/struct.QueueTrigger.html)
//!
//! The following Azure Functions output bindings are supported:
//!
//! * [HTTP output](bindings/struct.HttpResponse.html)
//! * [Queue message output](bindings/struct.QueueMessage.html)
//!
//! Eventually more bindings will be implemented, including custom binding data.
//!
//! # Examples
//!
//! Start by creating a new binary package:
//!
//! ```bash
//! $ cargo new --bin example
//! ```
//!
//! Edit `Cargo.toml` to include the following dependencies:
//!
//! ```toml
//! azure-functions = "0.1.4"
//! log = "0.4.2"
//! ```
//!
//! Azure Functions are implemented by applying a trigger attribute to a Rust function.
//!
//! For example, let's create `src/greet.rs` that implements a HTTP triggered function by
//! applying the `func` attribute:
//!
//! ```rust
//! # #![feature(use_extern_macros)] extern crate azure_functions;
//! # #[macro_use] extern crate log;
//! use azure_functions::func;
//! use azure_functions::bindings::{HttpRequest, HttpResponse};
//!
//! #[func]
//! #[binding(name = "request", auth_level = "anonymous")]
//! pub fn greet(request: &HttpRequest) -> HttpResponse {
//!     // Log the request on the Azure Functions Host
//!     info!("Request: {:?}", request);
//!
//!     // Return a formatted string as the response
//!     format!(
//!         "Hello from Rust, {}!",
//!         request.query_params().get("name").map_or("stranger", |x| x)
//!     ).into()
//! }
//! ```
//!
//! Replace the contents of `src/main.rs` with the following to register the function with
//! the Azure Functions Host:
//!
//! ```rust,ignore
//! #![feature(use_extern_macros)]
//!
//! #[macro_use]
//! extern crate log;
//! extern crate azure_functions;
//!
//! mod greet;
//!
//! // The main! macro generates an entrypoint for the binary
//! // Expects a list of Azure Functions to register with the Azure Functions host
//! azure_functions::main!{
//!     greet::greet
//! }
//! ```
//!
//! Run the application with the `--create <root>` option, where `<root>` is the path to
//! the desired Azure Functions application root directory:
//!
//! ```bash
//! $ export AzureWebJobsScriptRoot=path-to-root
//! $ cargo run -q -- --create $AzureWebJobsScriptRoot
//! ```
//!
//! Run the Azure Functions Host:
//!
//! ```bash
//! $ cd azure-functions-host/src/WebJobs.Script.WebHost
//! $ dotnet run
//! ```
//!
//! The above Azure Function can be invoked with `http://localhost:5000/api/greet?name=John`.
//!
//! The expected response would be `Hello from Rust, John!`.
#![feature(use_extern_macros)]
#![feature(proc_macro_mod)]
#![feature(proc_macro_gen)]
#![deny(missing_docs)]
#![deny(unused_extern_crates)]

extern crate azure_functions_codegen;
extern crate azure_functions_shared;
extern crate clap;
extern crate futures;
extern crate grpcio;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate tokio_threadpool;

#[doc(no_inline)]
pub use azure_functions_codegen::func;

#[doc(hidden)]
pub use azure_functions_shared::codegen;

mod cli;
mod logger;
mod registry;
mod util;

pub mod bindings;
pub mod http;
pub mod queue;
#[doc(hidden)]
pub mod rpc;
pub mod timer;
#[doc(no_inline)]
pub use azure_functions_codegen::main;
pub use azure_functions_shared::Context;

use futures::Future;
use registry::Registry;
use std::sync::{Arc, Mutex};

#[doc(hidden)]
pub fn worker_main(args: impl Iterator<Item = String>, functions: &[&'static codegen::Function]) {
    let matches = cli::create_app().get_matches_from(args);
    let registry = Arc::new(Mutex::new(Registry::new(functions)));

    if let Some(root) = matches.value_of("create") {
        cli::generate_functions_app(root, registry);
        return;
    }

    let client = rpc::Client::new(
        matches
            .value_of("worker_id")
            .expect("A worker id is required.")
            .to_owned(),
        matches
            .value_of("max_message_length")
            .map(|len| len.parse::<i32>().expect("Invalid maximum message length")),
    );

    let host = matches.value_of("host").expect("A host is required.");
    let port = matches
        .value_of("port")
        .map(|port| port.parse::<u32>().expect("Invalid port number"))
        .expect("Port number is required.");

    println!("Connecting to Azure Functions host at {}:{}.", host, port);

    client
        .connect(host, port)
        .and_then(|client| {
            println!(
                "Connected to Azure Functions host version {}.",
                client.host_version().unwrap()
            );

            client.process_all_messages(registry)
        })
        .wait()
        .unwrap();
}
