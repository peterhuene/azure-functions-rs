//! # Azure Functions for Rust
//!
//! The Azure Functions for Rust crate supports creating Azure Functions with Rust.
//!
//! The following Azure Functions trigger bindings are supported:
//!
//! * [Blob trigger](bindings/struct.BlobTrigger.html)
//! * [Cosmos DB trigger](bindings/struct.CosmosDbTrigger.html)
//! * [Durable Activity trigger](bindings/struct.DurableOrchestrationContext.html)
//! * [Durable Orchestration trigger](bindings/struct.DurableOrchestrationContext.html)
//! * [Event Grid trigger](bindings/struct.EventGridEvent.html)
//! * [Event Hub trigger](bindings/struct.EventHubTrigger.html)
//! * [Generic trigger](bindings/struct.GenericTrigger.html)
//! * [HTTP trigger](bindings/struct.HttpRequest.html)
//! * [Service Bus trigger](bindings/struct.ServiceBusTrigger.html)
//! * [Queue trigger](bindings/struct.QueueTrigger.html)
//! * [Timer trigger](bindings/struct.TimerInfo.html)
//!
//! The following Azure Functions input bindings are supported:
//!
//! * [Blob input](bindings/struct.Blob.html)
//! * [Cosmos DB input](bindings/struct.CosmosDbDocument.html)
//! * [Durable orchestration client input](bindings/struct.DurableOrchestrationClient.html)
//! * [Generic input](bindings/struct.GenericInput.html)
//! * [SignalR connection info input](bindings/struct.SignalRConnectionInfo.html)
//! * [Table input](bindings/struct.Table.html)
//!
//! The following Azure Functions output bindings are supported:
//!
//! * [Blob output](bindings/struct.Blob.html)
//! * [Cosmos DB output](bindings/struct.CosmosDbDocument.html)
//! * [Event Hub output](bindings/struct.EventHubMessage.html)
//! * [Generic output](bindings/struct.GenericOutput.html)
//! * [HTTP output](bindings/struct.HttpResponse.html)
//! * [Queue output](bindings/struct.QueueMessage.html)
//! * [SendGrid email message output](bindings/struct.SendGridMessage.html)
//! * [Service Bus output](bindings/struct.ServiceBusMessage.html)
//! * [SignalR group action output](bindings/struct.SignalRGroupAction.html)
//! * [SignalR message output](bindings/struct.SignalRMessage.html)
//! * [Table output](bindings/struct.Table.html)
//! * [Twilio SMS message output](bindings/struct.TwilioSmsMessage.html)
//!
//! Eventually more bindings will be implemented, including custom binding data.
//!
//! # Example
//!
//! Start by installing the Azure Functions for Rust SDK:
//!
//! ```bash
//! $ cargo install azure-functions-sdk
//! ```
//!
//! Create a new Azure Functions for Rust application:
//!
//! ```bash
//! $ cargo func new-app hello && cd hello
//! ```
//!
//! Create a HTTP-triggered function:
//!
//! ```bash
//! $ cargo func new http -n hello
//! ```
//!
//! This generates `src/functions/hello.rs` with the following contents:
//!
//! ```rust,ignore
//! use azure_functions::{
//!     bindings::{HttpRequest, HttpResponse},
//!     func,
//! };
//!
//! #[func]
//! pub fn hello(req: HttpRequest) -> HttpResponse {
//!     "Hello from Rust!".into()
//! }
//! ```
//!
//! Azure Functions are implemented by applying a `#[func]` attribute to a Rust function.
//!
//! Run the application with `cargo func run`:
//!
//! ```bash
//! $ cargo func run
//! ```
//!
//! The above Azure Function can be invoked with `http://localhost:8080/api/hello`.
//!
//! The expected response would be `Hello from Rust!`.
#![deny(unused_extern_crates)]
#![deny(missing_docs)]
#![cfg_attr(test, recursion_limit = "128")]
#![feature(async_await)]

#[doc(no_inline)]
pub use azure_functions_codegen::export;
#[doc(no_inline)]
pub use azure_functions_codegen::func;

#[doc(hidden)]
pub use azure_functions_shared::codegen;

mod backtrace;
mod commands;
mod logger;
mod registry;
mod util;
mod worker;

pub mod bindings;
pub mod blob;
pub mod context;
pub mod durable;
pub mod event_hub;
pub mod generic;
pub mod http;
pub mod send_grid;
pub mod signalr;
pub mod timer;
#[doc(no_inline)]
pub use azure_functions_shared::rpc;

use crate::commands::{Init, Run, SyncExtensions};
use crate::registry::Registry;
use clap::{App, AppSettings};

#[doc(hidden)]
pub trait IntoVec<T> {
    fn into_vec(self) -> Vec<T>;
}

#[doc(hidden)]
pub trait FromVec<T> {
    fn from_vec(vec: Vec<T>) -> Self;
}

/// The main entry point for the Azure Functions for Rust worker.
///
/// This entry point does not use any additional Azure Functions binding extensions.
///
/// # Examples
///
/// ```rust,ignore
/// mod example;
///
/// azure_functions::export! {
///     example::function,
/// }
///
/// fn main() {
///     azure_functions::worker_main(::std::env::args(), EXPORTS);
/// }
/// ```
pub fn worker_main(args: impl Iterator<Item = String>, functions: &[&'static codegen::Function]) {
    worker_main_with_extensions(args, functions, &[])
}

/// The main entry point for the Azure Functions for Rust worker.
///
/// This entry point uses additional Azure Function binding extensions.
///
/// # Examples
///
/// ```rust,ignore
/// fn main() {
///     azure_functions::worker_main_with_extensions(
///         ::std::env::args(),
///         functions::EXPORTS,
///         &[("Microsoft.Azure.WebJobs.Extensions.Kafka", "1.0.0-alpha")]
///     );
/// }
/// ```
pub fn worker_main_with_extensions(
    args: impl Iterator<Item = String>,
    functions: &[&'static codegen::Function],
    extensions: &[(&str, &str)],
) {
    let registry = Registry::new(functions);

    let app = App::new("Azure Functions for Rust worker")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Implements the Azure Functions for Rust worker.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(Init::create_subcommand())
        .subcommand(SyncExtensions::create_subcommand())
        .subcommand(Run::create_subcommand());

    if let Err(e) = match app.get_matches_from(args).subcommand() {
        ("init", Some(args)) => Init::from(args).execute(registry, extensions),
        ("sync-extensions", Some(args)) => SyncExtensions::from(args).execute(registry, extensions),
        ("run", Some(args)) => Run::from(args).execute(registry),
        _ => panic!("expected a subcommand."),
    } {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
