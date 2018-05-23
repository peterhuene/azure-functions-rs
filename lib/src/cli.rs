use clap::{App, Arg};
use registry::Registry;
use serde::Serialize;
use serde_json::Serializer;
use std::env::{current_dir, current_exe};
use std::fs;
use std::sync::{Arc, Mutex};

pub fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Azure Function Language Worker for Rust")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Provides an Azure Function Worker for functions written in Rust.")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOST")
                .help("The hostname of the Azure Functions Host.")
                .conflicts_with("create")
                .required_unless("create"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("The port of the Azure Functions Host.")
                .conflicts_with("create")
                .required_unless("create"),
        )
        .arg(
            Arg::with_name("worker_id")
                .long("workerId")
                .value_name("WORKER_ID")
                .help("The worker ID to use when registering with the Azure Functions Host.")
                .conflicts_with("create")
                .required_unless("create"),
        )
        .arg(
            Arg::with_name("request_id")
                .long("requestId")
                .value_name("REQUEST_ID")
                .help("The request ID to use when communicating with the Azure Functions Host.")
                .hidden(true)
                .conflicts_with("create")
                .required_unless("create"),
        )
        .arg(
            Arg::with_name("max_message_length")
                .long("grpcMaxMessageLength")
                .value_name("MAXIMUM")
                .help("The maximum message length to use for gRPC messages.")
                .conflicts_with("create")
                .required_unless("create"),
        )
        .arg(
            Arg::with_name("create")
                .long("create")
                .value_name("APP_ROOT")
                .help("Creates the Azure Functions App at the given root directory.\nCannot be used with other options."),
        )
}

pub fn generate_functions_app(root: &str, registry: Arc<Mutex<Registry>>) {
    const FUNCTION_FILE: &'static str = "function.json";
    const RUST_SCRIPT_FILE: &'static str = "run.rs";

    let root = current_dir()
        .expect("failed to get current directory")
        .join(root);

    if root.exists() {
        println!(
            "Using existing Azure Functions application at '{}'.",
            root.display()
        );
    } else {
        println!(
            "Creating Azure Functions application at '{}'.",
            root.display()
        );

        fs::create_dir_all(&root).expect(&format!(
            "Failed to create Azure Functions application directory '{}'",
            root.display()
        ));
    }

    let host_json = root.join("host.json");
    if !host_json.exists() {
        println!(
            "Creating empty host configuration file '{}'.",
            host_json.display()
        );
        fs::write(&host_json, "{}").expect(&format!("Failed to create '{}'", host_json.display()));
    }

    println!("Copying current worker executable.");
    fs::copy(
        current_exe().expect("Failed to determine the path to the current executable"),
        root.join("rust_worker"),
    ).expect("Failed to copy worker executable");

    for entry in fs::read_dir(&root).expect("failed to read script root directory") {
        let path = root.join(entry.expect("failed to read script root entry").path());
        if !path.is_dir() || !path.join(RUST_SCRIPT_FILE).exists() {
            continue;
        }

        println!("Deleting existing function directory '{}'.", path.display());

        fs::remove_dir_all(&path).expect(&format!(
            "Failed to delete function directory '{}",
            path.display()
        ));
    }

    for (name, info) in registry.lock().unwrap().iter() {
        let function_dir = root.join(name);
        fs::create_dir(&function_dir).expect(&format!(
            "Failed to create function directory '{}'",
            function_dir.display()
        ));

        let script_file = function_dir.join(RUST_SCRIPT_FILE);
        println!(
            "Creating script file '{}' for Azure Function '{}'.",
            script_file.display(),
            name
        );
        fs::write(
            &script_file,
            "// This file is intentionally empty.\n// It is needed by Azure Functions Host to register the Azure function."
        ).expect(&format!("Failed to create '{}'", script_file.display()));

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
