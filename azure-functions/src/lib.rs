//! # Azure Functions for Rust
//!
//! The Azure Functions for Rust crate supports creating Azure Functions with Rust.
//!
//! The following Azure Functions trigger bindings are supported:
//!
//! * [Blob triggers](bindings/struct.BlobTrigger.html)
//! * [HTTP triggers](bindings/struct.HttpRequest.html)
//! * [Queue triggers](bindings/struct.QueueTrigger.html)
//! * [Timer triggers](bindings/struct.TimerInfo.html)
//!
//! The following Azure Functions input bindings are supported:
//!
//! * [Blob input](bindings/struct.Blob.html)
//! * [Table input](bindings/struct.Table.html)
//!
//! The following Azure Functions output bindings are supported:
//!
//! * [Blob output](bindings/struct.Blob.html)
//! * [HTTP output](bindings/struct.HttpResponse.html)
//! * [Queue message output](bindings/struct.QueueMessage.html)
//! * [Table output](bindings/struct.Table.html)
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
//! azure-functions = "0.2.0"
//! log = "0.4.2"
//! ```
//!
//! Azure Functions are implemented by applying a `#[func]` attribute to a Rust function.
//!
//! For example, let's create `src/greet.rs` that implements a HTTP triggered function:
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
//! Initialize the application with the `init` command, where `$AzureWebJobsScriptRoot` is
//! the desired Azure Functions script root directory:
//!
//! ```bash
//! $ export AzureWebJobsScriptRoot=path-to-root
//! $ cargo run -q -- init --worker-path /tmp/example/rust_worker --script-root /tmp/example/root
//! ```
//!
//! Run the [Azure Functions Host](https://github.com/azure/azure-functions-host):
//!
//! ```bash
//! $ cd azure-functions-host/src/WebJobs.Script.WebHost
//! $ PATH=/tmp/example:$PATH AzureWebJobsScriptRoot=/tmp/example/root dotnet run
//! ```
//!
//! The above Azure Function can be invoked with `http://localhost:5000/api/greet?name=John`.
//!
//! The expected response would be `Hello from Rust, John!`.
#![feature(rust_2018_preview)]
#![feature(in_band_lifetimes)]
#![feature(proc_macro_mod)]
#![feature(proc_macro_gen)]
#![deny(missing_docs)]
#![deny(unused_extern_crates)]
#![cfg_attr(test, recursion_limit = "128")]

extern crate azure_functions_codegen;
extern crate azure_functions_shared;
extern crate clap;
extern crate futures;
extern crate grpcio;
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate tokio_threadpool;
#[cfg(test)]
#[macro_use(matches)]
extern crate matches;

#[doc(no_inline)]
pub use azure_functions_codegen::func;

#[doc(hidden)]
pub use azure_functions_shared::codegen;

mod cli;
mod logger;
mod registry;
mod util;

pub mod bindings;
pub mod blob;
pub mod http;
#[doc(hidden)]
pub mod rpc;
pub mod timer;
#[doc(no_inline)]
pub use azure_functions_codegen::main;
pub use azure_functions_shared::Context;

use futures::Future;
use registry::Registry;
use serde::Serialize;
use serde_json::Serializer;
use std::env::{current_dir, current_exe};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// This is a workaround to the issue that `file!` expands to be workspace-relative
// and cargo does not have an environment variable for the workspace directory.
// Thus, this walks up the manifest directory until it hits "src" in the file's path.
// This function is sensitive to cargo and rustc changes.
fn get_source_file_path(manifest_dir: &Path, file: &Path) -> PathBuf {
    let mut manifest_dir = Path::new(manifest_dir);
    for component in file.components() {
        if component.as_os_str() == "src" {
            break;
        }
        manifest_dir = manifest_dir
            .parent()
            .expect("expected another parent for the manifest directory");
    }

    manifest_dir.join(file)
}

fn has_rust_files(directory: &Path) -> bool {
    fs::read_dir(directory)
        .expect(&format!(
            "failed to read directory '{}'",
            directory.display()
        )).any(|p| {
            if let Ok(p) = p {
                let p = p.path();
                p.is_file() && p.extension().map(|x| x == "rs").unwrap_or(false)
            } else {
                false
            }
        })
}

fn initialize_app(worker_path: &str, script_root: &str, registry: Arc<Mutex<Registry<'static>>>) {
    const FUNCTION_FILE: &'static str = "function.json";

    let script_root = current_dir()
        .expect("failed to get current directory")
        .join(script_root);

    if script_root.exists() {
        println!(
            "Using existing Azure Functions application at '{}'.",
            script_root.display()
        );
    } else {
        println!(
            "Creating Azure Functions application at '{}'.",
            script_root.display()
        );

        fs::create_dir_all(&script_root).expect(&format!(
            "Failed to create Azure Functions application directory '{}'",
            script_root.display()
        ));
    }

    let host_json = script_root.join("host.json");
    if !host_json.exists() {
        println!(
            "Creating empty host configuration file '{}'.",
            host_json.display()
        );
        fs::write(&host_json, "{}").expect(&format!("Failed to create '{}'", host_json.display()));
    }

    let worker_dir = Path::new(worker_path)
        .parent()
        .expect("expected to get a parent of the worker path");
    fs::create_dir_all(&worker_dir).expect(&format!(
        "Failed to create directory for worker executable '{}'",
        worker_dir.display()
    ));

    println!("Copying current worker executable to '{}'.", worker_path);
    fs::copy(
        current_exe().expect("Failed to determine the path to the current executable"),
        worker_path,
    ).expect("Failed to copy worker executable");

    for entry in fs::read_dir(&script_root).expect("failed to read script root directory") {
        let path = script_root.join(entry.expect("failed to read script root entry").path());
        if !path.is_dir() || !has_rust_files(&path) {
            continue;
        }

        println!(
            "Deleting existing Rust function directory '{}'.",
            path.display()
        );

        fs::remove_dir_all(&path).expect(&format!(
            "Failed to delete function directory '{}",
            path.display()
        ));
    }

    for (name, info) in registry.lock().unwrap().iter() {
        let function_dir = script_root.join(name);
        fs::create_dir(&function_dir).expect(&format!(
            "Failed to create function directory '{}'",
            function_dir.display()
        ));

        let source_file = get_source_file_path(
            Path::new(
                info.manifest_dir
                    .as_ref()
                    .expect("Functions should have a manifest directory.")
                    .as_ref(),
            ),
            Path::new(
                info.file
                    .as_ref()
                    .expect("Functions should have a file.")
                    .as_ref(),
            ),
        );

        let destination_file = function_dir.join(
            source_file
                .file_name()
                .expect("expected the source file to have a file name"),
        );

        if source_file.is_file() {
            println!(
                "Copying source file '{}' to '{}' for Azure Function '{}'.",
                source_file.display(),
                destination_file.display(),
                name
            );
            fs::copy(&source_file, destination_file).expect(&format!(
                "Failed to copy source file '{}'",
                source_file.display()
            ));
        } else {
            println!(
                "Creating empty source file '{}' for Azure Function '{}'.",
                destination_file.display(),
                name
            );
            fs::write(
                &destination_file,
                "// This file is intentionally empty.\n\
                 // The original source file was not available when the Functions Application was initialized.\n"
            ).expect(&format!("Failed to create '{}'", destination_file.display()));
        }

        let function_json = function_dir.join(FUNCTION_FILE);
        println!(
            "Creating function configuration file '{}' for Azure Function '{}'.",
            function_json.display(),
            name
        );

        let mut output = fs::File::create(&function_json)
            .expect(&format!("Failed to create '{}'", function_json.display()));

        info.serialize(&mut Serializer::pretty(&mut output))
            .expect(&format!(
                "Failed to serialize metadata for function '{}'",
                name
            ));
    }
}

fn run_worker(
    worker_id: &str,
    host: &str,
    port: u32,
    max_message_length: Option<i32>,
    registry: Arc<Mutex<Registry<'static>>>,
) {
    let client = rpc::Client::new(worker_id.to_string(), max_message_length);

    println!("Connecting to Azure Functions host at {}:{}.", host, port);

    client
        .connect(host, port)
        .and_then(|client| {
            println!(
                "Connected to Azure Functions host version {}.",
                client.host_version().unwrap()
            );

            client.process_all_messages(registry)
        }).wait()
        .unwrap();
}

#[doc(hidden)]
pub fn worker_main(args: impl Iterator<Item = String>, functions: &[&'static codegen::Function]) {
    let matches = cli::create_app().get_matches_from(args);
    let registry = Arc::new(Mutex::new(Registry::new(functions)));

    if let Some(matches) = matches.subcommand_matches("init") {
        initialize_app(
            matches
                .value_of("worker_path")
                .expect("A binary path is required."),
            matches
                .value_of("script_root")
                .expect("A script root is required."),
            registry,
        );
        return;
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        run_worker(
            matches
                .value_of("worker_id")
                .expect("A worker id is required."),
            matches.value_of("host").expect("A host is required."),
            matches
                .value_of("port")
                .map(|port| port.parse::<u32>().expect("Invalid port number"))
                .expect("A port number is required."),
            matches
                .value_of("max_message_length")
                .map(|len| len.parse::<i32>().expect("Invalid maximum message length")),
            registry,
        );
        return;
    }

    cli::create_app().print_help().unwrap();
    println!();
}
