//! # Azure Functions for Rust
//!
//! The Azure Functions for Rust crate supports creating Azure Functions with Rust.
//!
//! The following Azure Functions trigger bindings are supported:
//!
//! * [Blob trigger](bindings/struct.BlobTrigger.html)
//! * [Cosmos DB trigger](bindings/struct.CosmosDbTrigger.html)
//! * [Event Grid trigger](bindings/struct.EventGridEvent.html)
//! * [Event Hub trigger](bindings/struct.EventHubTrigger.html)
//! * [HTTP trigger](bindings/struct.HttpRequest.html)
//! * [Service Bus trigger](bindings/struct.ServiceBusTrigger.html)
//! * [Queue trigger](bindings/struct.QueueTrigger.html)
//! * [Timer trigger](bindings/struct.TimerInfo.html)
//!
//! The following Azure Functions input bindings are supported:
//!
//! * [Blob input](bindings/struct.Blob.html)
//! * [Cosmos DB input](bindings/struct.CosmosDbDocument.html)
//! * [SignalR connection info input](bindings/struct.SignalRConnectionInfo.html)
//! * [Table input](bindings/struct.Table.html)
//!
//! The following Azure Functions output bindings are supported:
//!
//! * [Blob output](bindings/struct.Blob.html)
//! * [Cosmos DB output](bindings/struct.CosmosDbDocument.html)
//! * [Event Hub output](bindings/struct.EventHubMessage.html)
//! * [HTTP output](bindings/struct.HttpResponse.html)
//! * [Queue output](bindings/struct.QueueMessage.html)
//! * [Service Bus output](bindings/struct.ServiceBusMessage.html)
//! * [SignalR group action output](bindings/struct.SignalRGroupAction.html)
//! * [SignalR message output](bindings/struct.SignalRMessage.html)
//! * [Table output](bindings/struct.Table.html)
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

#[doc(no_inline)]
pub use azure_functions_codegen::func;

#[doc(hidden)]
pub use azure_functions_shared::codegen;

mod backtrace;
mod cli;
mod commands;
mod logger;
mod registry;
mod util;

pub mod bindings;
pub mod blob;
pub mod event_hub;
pub mod http;
pub mod signalr;
pub mod timer;
#[doc(no_inline)]
pub use azure_functions_codegen::export;
pub use azure_functions_shared::{rpc, Context};

use crate::registry::Registry;
use std::env::current_dir;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use xml::writer::XmlEvent;
use xml::EmitterConfig;

#[doc(hidden)]
pub trait IntoVec<T> {
    fn into_vec(self) -> Vec<T>;
}

#[doc(hidden)]
pub trait FromVec<T> {
    fn from_vec(vec: Vec<T>) -> Self;
}

fn write_property(writer: &mut xml::EventWriter<&mut fs::File>, name: &str, value: &str) {
    writer.write(XmlEvent::start_element(name)).unwrap();
    writer.write(XmlEvent::characters(value)).unwrap();
    writer.write(XmlEvent::end_element()).unwrap();
}

fn write_extensions_project_file(path: &Path, registry: &Registry<'static>) {
    let mut project_file =
        fs::File::create(path).expect("Failed to create extensions project file.");

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut project_file);

    writer
        .write(XmlEvent::start_element("Project").attr("Sdk", "Microsoft.NET.Sdk"))
        .unwrap();

    writer
        .write(XmlEvent::start_element("PropertyGroup"))
        .unwrap();

    write_property(&mut writer, "TargetFramework", "netstandard2.0");
    write_property(&mut writer, "CopyBuildOutputToPublishDirectory", "false");
    write_property(&mut writer, "CopyOutputSymbolsToPublishDirectory", "false");
    write_property(&mut writer, "GenerateDependencyFile", "false");

    writer.write(XmlEvent::end_element()).unwrap();

    writer.write(XmlEvent::start_element("ItemGroup")).unwrap();

    for extension in registry.iter_binding_extensions() {
        writer
            .write(
                XmlEvent::start_element("PackageReference")
                    .attr("Include", extension.0)
                    .attr("Version", extension.1),
            )
            .unwrap();
        writer.write(XmlEvent::end_element()).unwrap();
    }

    writer.write(XmlEvent::end_element()).unwrap();
    writer.write(XmlEvent::end_element()).unwrap();
}

fn write_generator_project_file(path: &Path) {
    let mut project_file =
        fs::File::create(path).expect("Failed to create generator project file.");

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut project_file);

    writer
        .write(XmlEvent::start_element("Project").attr("Sdk", "Microsoft.NET.Sdk"))
        .unwrap();

    writer
        .write(XmlEvent::start_element("PropertyGroup"))
        .unwrap();

    write_property(&mut writer, "TargetFramework", "netstandard2.0");

    writer.write(XmlEvent::end_element()).unwrap();

    writer.write(XmlEvent::start_element("ItemGroup")).unwrap();

    writer
        .write(
            XmlEvent::start_element("PackageReference")
                .attr(
                    "Include",
                    "Microsoft.Azure.WebJobs.Script.ExtensionsMetadataGenerator",
                )
                .attr("Version", "1.0.1")
                .attr("PrivateAssets", "all"),
        )
        .unwrap();

    writer.write(XmlEvent::end_element()).unwrap();
    writer.write(XmlEvent::end_element()).unwrap();
    writer.write(XmlEvent::end_element()).unwrap();
}

fn sync_extensions(script_root: &str, verbose: bool, registry: Registry<'static>) {
    if !registry.has_binding_extensions() {
        if verbose {
            println!("No binding extensions are needed.");
        }
        return;
    }

    let temp_dir = TempDir::new().expect("failed to create temporary directory");
    let extensions_project_path = temp_dir.path().join("extensions.csproj");
    let metadata_project_path = temp_dir.path().join("metadata.csproj");
    let output_directory = current_dir()
        .expect("failed to get current directory")
        .join(script_root);

    write_extensions_project_file(&extensions_project_path, &registry);
    write_generator_project_file(&metadata_project_path);

    if verbose {
        println!("Restoring extension assemblies...");
    }

    let status = Command::new("dotnet")
        .args(&[
            "publish",
            "-c",
            "Release",
            "-o",
            output_directory.join("bin").to_str().unwrap(),
            "/v:q",
            "/nologo",
            extensions_project_path.to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn dotnet: {}", e));

    if !status.success() {
        panic!(
            "failed to restore extensions: dotnet returned non-zero exit code {}.",
            status.code().unwrap()
        );
    }

    if verbose {
        println!("Generating extension metadata...");
    }

    let status = Command::new("dotnet")
        .args(&[
            "msbuild",
            "/t:_GenerateFunctionsExtensionsMetadataPostPublish",
            "/v:q",
            "/nologo",
            "/restore",
            "-p:Configuration=Release",
            &format!("-p:PublishDir={}/", output_directory.to_str().unwrap()),
            metadata_project_path.to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn dotnet: {}", e));

    if !status.success() {
        panic!(
            "failed to generate extension metadata: dotnet returned non-zero exit code {}.",
            status.code().unwrap()
        );
    }
}

/// The main entry point for the Azure Functions for Rust worker.
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
pub fn worker_main(args: impl Iterator<Item = String>, functions: &[&'static codegen::Function]) {
    let matches = cli::create_app().get_matches_from(args);
    let registry = Registry::new(functions);

    if let Some(matches) = matches.subcommand_matches("sync-extensions") {
        sync_extensions(
            matches
                .value_of("script_root")
                .expect("A script root is required."),
            matches.is_present("verbose"),
            registry,
        );
        return;
    }

    if let Err(e) = match matches
        //.get_or_insert_with(|| create_app().get_matches_from(env::args().skip(1)))
        .subcommand()
    {
        ("init", Some(args)) => commands::Init::from(args).execute(registry),
        ("run", Some(args)) => commands::Run::from(args).execute(registry),
        _ => panic!("expected a subcommand."),
    } {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
