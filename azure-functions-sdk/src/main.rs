//! # Azure Functions for Rust SDK
//!
//! The Azure Functions for Rust SDK is a cargo extension for creating Azure Functions applications
//!
//! Start by installing the Azure Functions for Rust SDK
//!
//! ```bash
//! $ cargo install azure-functions-sdk
//! ```
//!
//! Next, create a new Azure Functions application:
//!
//! ```bash
//! $ cargo func new-app hello
//! ```
//!
//! Azure Functions are implemented by applying a `#[func]` attribute to a Rust function.
//!
//! For example, let's create `src/functions/hello.rs` that implements a HTTP triggered function:
//!
//! ```rust,ignore
//! use azure_functions::func;
//! use azure_functions::bindings::{HttpRequest, HttpResponse};
//!
//! #[func]
//! #[binding(name = "request", auth_level = "anonymous")]
//! pub fn hello(request: &HttpRequest) -> HttpResponse {
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
//! Export the function in `src/functions/mod.rs`:
//!
//! ```rust,ignore
//! mod hello;
//!
//! pub const FUNCTIONS: &[&azure_functions::codegen::Function] = azure_functions::export!{
//!   hello::hello
//! };
//! ```
//!
//! Run the application:
//!
//! ```bash
//! $ cargo func run
//! ```
//!
//! Now invoke the function using cURL from a different terminal session:
//!
//! ```bash
//! $ curl http://localhost:8080/api/hello\?name\=John
//! Hello from Rust, John!
//! ```
#![feature(in_band_lifetimes)]
#![deny(missing_docs)]
#![deny(unused_extern_crates)]

#[macro_use]
extern crate serde_json;

mod commands;
mod util;

use crate::commands::{Build, NewApp, Run};
use clap::{App, AppSettings};
use colored::Colorize;
use std::env;
use std::process;

fn create_app() -> App<'a, 'b> {
    App::new("Azure Functions for Rust")
        .bin_name("cargo func")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Azure Functions for Rust Developer Tools")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::NoBinaryName)
        .subcommand(Build::create_subcommand())
        .subcommand(NewApp::create_subcommand())
        .subcommand(Run::create_subcommand())
}

fn print_error_and_exit(message: &str) {
    eprintln!("{}: {}", "error".red().bold(), message);
    process::exit(1);
}

fn main() {
    // Support both cargo-func <command> and cargo-func func <command>
    // This enables running both `cargo-func` and `cargo func`, which passes the `func` command down
    let mut matches = None;
    if let Some(first) = env::args().nth(1) {
        if first == "func" {
            matches = Some(create_app().get_matches_from(env::args().skip(2)));
        }
    }

    if let Err(e) = match matches
        .get_or_insert_with(|| create_app().get_matches_from(env::args().skip(1)))
        .subcommand()
    {
        ("new-app", Some(args)) => NewApp::from(args).execute(),
        ("build", Some(args)) => Build::from(args).execute(),
        ("run", Some(args)) => Run::from(args).execute(),
        _ => panic!("expected a subcommand."),
    } {
        print_error_and_exit(&e);
    }
}
