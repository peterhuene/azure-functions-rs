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
//! $ cargo func new-app hello && cd hello
//! ```
//!
//! Create a HTTP-triggered function:
//!
//! ```bash
//! $ cargo func new http hello
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
#![deny(missing_docs)]
#![deny(unused_extern_crates)]
#![warn(clippy::use_self)]
#![warn(clippy::option_map_unwrap_or)]
#![warn(clippy::option_map_unwrap_or_else)]

mod commands;
mod util;

use clap::{App, AppSettings};
use colored::Colorize;

use std::{env, process};

use crate::commands::{New, NewApp, Run};

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Azure Functions for Rust")
        .bin_name("cargo func")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Azure Functions for Rust Developer Tools")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::NoBinaryName)
        .subcommand(NewApp::create_subcommand())
        .subcommand(Run::create_subcommand())
        .subcommand(New::create_subcommand())
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
        ("run", Some(args)) => Run::from(args).execute(),
        ("new", Some(args)) => New::from(args).execute(),
        _ => panic!("expected a subcommand."),
    } {
        print_error_and_exit(&e);
    }
}
