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
//! pub fn hello(req: &HttpRequest) -> HttpResponse {
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
//! The above Azure Function can be invoked with `http://localhost:8080/api/hello?name=Peter`.
//!
//! The expected response would be `Hello from Rust, Peter!`.
#![deny(unused_extern_crates)]
#![cfg_attr(test, recursion_limit = "128")]

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use(matches)]
extern crate matches;
#[macro_use]
extern crate lazy_static;

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
pub use azure_functions_codegen::export;
pub use azure_functions_shared::Context;

use crate::registry::Registry;
use futures::Future;
use serde::Serialize;
use serde_json::{to_string_pretty, Serializer};
use std::env::{current_dir, current_exe};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use xml::writer::XmlEvent;
use xml::EmitterConfig;

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
        .unwrap_or_else(|_| panic!("failed to read directory '{}'", directory.display()))
        .any(|p| match p {
            Ok(p) => {
                let p = p.path();
                p.is_file() && p.extension().map(|x| x == "rs").unwrap_or(false)
            }
            _ => false,
        })
}

fn create_script_root(script_root: &Path, verbose: bool) {
    if script_root.exists() {
        if verbose {
            println!(
                "Using existing Azure Functions application at '{}'.",
                script_root.display()
            );
        }
    } else {
        if verbose {
            println!(
                "Creating Azure Functions application at '{}'.",
                script_root.display()
            );
        }

        fs::create_dir_all(&script_root).unwrap_or_else(|_| {
            panic!(
                "Failed to create Azure Functions application directory '{}'",
                script_root.display()
            )
        });
    }
}

fn create_host_file(script_root: &Path, verbose: bool) {
    let host_json = script_root.join("host.json");
    if host_json.exists() {
        return;
    }

    if verbose {
        println!(
            "Creating host configuration file '{}'.",
            host_json.display()
        );
    }

    fs::write(
        &host_json,
        to_string_pretty(&json!(
        {
            "version": "2.0",
            "logging": {
                "logLevel": {
                    "default": "Warning"
                }
            }
        }))
        .unwrap(),
    )
    .unwrap_or_else(|_| panic!("Failed to create '{}'", host_json.display()));
}

fn create_local_settings_file(script_root: &Path, worker_dir: &Path, verbose: bool) {
    let settings = script_root.join("local.settings.json");
    if settings.exists() {
        return;
    }

    if verbose {
        println!("Creating local settings file '{}'.", settings.display());
    }

    fs::write(
        &settings,
        to_string_pretty(&json!(
        {
            "IsEncrypted": false,
            "Values": {
                "FUNCTIONS_WORKER_RUNTIME": "Rust",
                "languageWorkers:workersDirectory": worker_dir.parent().unwrap()
            },
            "ConnectionStrings": {
            }
        }))
        .unwrap(),
    )
    .unwrap_or_else(|_| panic!("Failed to create '{}'", settings.display()));
}

fn create_worker_dir(script_root: &Path, verbose: bool) -> PathBuf {
    let worker_dir = script_root.join("workers").join("rust");

    if verbose {
        println!("Creating worker directory '{}'.", worker_dir.display());
    }

    fs::create_dir_all(&worker_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create directory for worker executable '{}'",
            worker_dir.display()
        )
    });

    worker_dir
}

fn copy_worker_executable(worker_dir: &Path, verbose: bool) {
    if verbose {
        println!(
            "Copying current worker executable to '{}'.",
            worker_dir.display()
        );
    }

    fs::copy(
        current_exe().expect("Failed to determine the path to the current executable"),
        worker_dir.join(if cfg!(windows) {
            "rust_worker.exe"
        } else {
            "rust_worker"
        }),
    )
    .expect("Failed to copy worker executable");
}

fn create_worker_config_file(worker_dir: &Path, verbose: bool) {
    let config = worker_dir.join("worker.config.json");
    if config.exists() {
        return;
    }

    if verbose {
        println!("Creating worker config file '{}'.", config.display());
    }

    fs::write(
        &config,
        to_string_pretty(&json!(
        {
            "description":{
                "language": "Rust",
                "extensions": [".rs"],
                "defaultExecutablePath": "workers/rust/rust_worker",
                "arguments": ["run"]
            }
        }))
        .unwrap(),
    )
    .unwrap_or_else(|_| panic!("Failed to create '{}'", config.display()));
}

fn delete_existing_function_directories(script_root: &Path, verbose: bool) {
    for entry in fs::read_dir(&script_root).expect("failed to read script root directory") {
        let path = script_root.join(entry.expect("failed to read script root entry").path());
        if !path.is_dir() || !has_rust_files(&path) {
            continue;
        }

        if verbose {
            println!(
                "Deleting existing Rust function directory '{}'.",
                path.display()
            );
        }

        fs::remove_dir_all(&path)
            .unwrap_or_else(|_| panic!("Failed to delete function directory '{}", path.display()));
    }
}

fn create_function_directory(script_root: &Path, function_name: &str, verbose: bool) -> PathBuf {
    let function_dir = script_root.join(function_name);

    if verbose {
        println!("Creating function directory '{}'.", function_dir.display());
    }

    fs::create_dir(&function_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create function directory '{}'",
            function_dir.display()
        )
    });

    function_dir
}

fn copy_source_file(function_dir: &Path, source_file: &Path, function_name: &str, verbose: bool) {
    let destination_file = function_dir.join(
        source_file
            .file_name()
            .expect("expected the source file to have a file name"),
    );

    if source_file.is_file() {
        if verbose {
            println!(
                "Copying source file '{}' to '{}' for Azure Function '{}'.",
                source_file.display(),
                destination_file.display(),
                function_name
            );
        }

        fs::copy(&source_file, destination_file)
            .unwrap_or_else(|_| panic!("Failed to copy source file '{}'", source_file.display()));
    } else {
        if verbose {
            println!(
                "Creating empty source file '{}' for Azure Function '{}'.",
                destination_file.display(),
                function_name
            );
        }

        fs::write(
                &destination_file,
                "// This file is intentionally empty.\n\
                 // The original source file was not available when the Functions Application was initialized.\n"
            ).unwrap_or_else(|_| panic!("Failed to create '{}'", destination_file.display()));
    }
}

fn create_function_config_file(
    function_dir: &Path,
    info: &'static codegen::Function,
    verbose: bool,
) {
    let function_json = function_dir.join("function.json");

    if verbose {
        println!(
            "Creating function configuration file '{}' for Azure Function '{}'.",
            function_json.display(),
            info.name
        );
    }

    let mut output = fs::File::create(&function_json)
        .unwrap_or_else(|_| panic!("Failed to create '{}'", function_json.display()));

    info.serialize(&mut Serializer::pretty(&mut output))
        .unwrap_or_else(|_| panic!("Failed to serialize metadata for function '{}'", info.name));
}

fn initialize_script_root(
    script_root: &str,
    sync: bool,
    verbose: bool,
    registry: &Arc<Mutex<Registry<'static>>>,
) {
    let script_root = current_dir()
        .expect("failed to get current directory")
        .join(script_root);

    create_script_root(&script_root, verbose);

    create_host_file(&script_root, verbose);

    let worker_dir = create_worker_dir(&script_root, verbose);

    create_local_settings_file(&script_root, &worker_dir, verbose);

    copy_worker_executable(&worker_dir, verbose);

    create_worker_config_file(&worker_dir, verbose);

    delete_existing_function_directories(&script_root, verbose);

    for (name, info) in registry.lock().unwrap().iter() {
        let function_dir = create_function_directory(&script_root, name, verbose);

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

        copy_source_file(&function_dir, &source_file, name, verbose);

        create_function_config_file(&function_dir, info, verbose);
    }

    if sync {
        sync_extensions(script_root.to_str().unwrap(), verbose, &registry);
    }
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

fn sync_extensions(script_root: &str, verbose: bool, registry: &Arc<Mutex<Registry<'static>>>) {
    let reg = registry.lock().unwrap();

    if !reg.has_binding_extensions() {
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

    write_extensions_project_file(&extensions_project_path, &reg);
    write_generator_project_file(&metadata_project_path);

    if verbose {
        println!("Restoring extension assemblies...");
    }

    if !Command::new("dotnet")
        .args(&[
            "publish",
            "/v:q",
            "/nologo",
            "-c",
            "Release",
            "-o",
            output_directory.join("bin").to_str().unwrap(),
            extensions_project_path.to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .status()
        .map_err(|e| format!("failed to spawn dotnet: {}", e))
        .unwrap()
        .success()
    {
        panic!("failed to restore extension assemblies.");
    }

    if verbose {
        println!("Generating extension metadata...");
    }

    if !Command::new("dotnet")
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
        .map_err(|e| format!("failed to spawn dotnet: {}", e))
        .unwrap()
        .success()
    {
        panic!("failed to generate extension metadata.");
    }
}

fn run_worker(
    worker_id: &str,
    host: &str,
    port: u32,
    max_message_length: Option<i32>,
    registry: &Arc<Mutex<Registry<'static>>>,
) {
    ctrlc::set_handler(|| {}).expect("failed setting SIGINT handler");

    let client = rpc::Client::new(worker_id.to_string(), max_message_length);

    println!("Connecting to Azure Functions host at {}:{}.", host, port);

    client
        .connect(host, port)
        .and_then(|client| {
            println!(
                "Connected to Azure Functions host version {}.",
                client.host_version().unwrap()
            );

            client.process_all_messages(&registry)
        })
        .wait()
        .unwrap();
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
    let registry = Arc::new(Mutex::new(Registry::new(functions)));

    if let Some(matches) = matches.subcommand_matches("init") {
        initialize_script_root(
            matches
                .value_of("script_root")
                .expect("A script root is required."),
            matches.is_present("sync"),
            matches.is_present("verbose"),
            &registry,
        );
        return;
    }

    if let Some(matches) = matches.subcommand_matches("sync-extensions") {
        sync_extensions(
            matches
                .value_of("script_root")
                .expect("A script root is required."),
            matches.is_present("verbose"),
            &registry,
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
            &registry,
        );
        return;
    }

    panic!("expected a subcommand.");
}
