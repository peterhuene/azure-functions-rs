use clap::{App, AppSettings, Arg, SubCommand};
use std::path::Path;

pub fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Azure Functions for Rust worker")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Implements the Azure Functions for Rust worker.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("init")
                .about("Initializes the Azure Functions application script root.")
                .arg(
                    Arg::with_name("script_root")
                        .long("script-root")
                        .value_name("SCRIPT_ROOT")
                        .help("The script root directory to initialize the application in.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("local_settings")
                        .long("local-settings")
                        .value_name("SETTINGS_FILE")
                        .help("The path to the local settings file to use. Defaults to the `local.settings.json` file in the directory containing `Cargo.toml`, if present.")
                        .validator(|v| {
                            if Path::new(&v).is_file() {
                                Ok(())
                            } else {
                                Err(format!("local settings file '{}' does not exist", v))
                            }
                        })
                )
                .arg(
                    Arg::with_name("sync_extensions")
                        .long("sync-extensions")
                        .short("s")
                        .help("Synchronize the Azure Function binding extensions.")
                )
                .arg(
                    Arg::with_name("no_debug_info")
                        .long("--no-debug-info")
                        .help("Do not copy debug information for the worker executable.")
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Use verbose output.")
                )
        )
        .subcommand(
            SubCommand::with_name("sync-extensions")
                .about("Synchronizes the Azure Function binding extensions used by the worker.")
                .arg(
                    Arg::with_name("script_root")
                        .long("script-root")
                        .value_name("SCRIPT_ROOT")
                        .help("The script root to synchronize the binding extensions for.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Use verbose output.")
                )
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs the Rust language worker.")
                .arg(
                    Arg::with_name("host")
                        .long("host")
                        .value_name("HOST")
                        .help("The hostname of the Azure Functions Host.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .value_name("PORT")
                        .help("The port of the Azure Functions Host.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("worker_id")
                        .long("workerId")
                        .value_name("WORKER_ID")
                        .help("The worker ID to use when registering with the Azure Functions Host.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("request_id")
                        .long("requestId")
                        .value_name("REQUEST_ID")
                        .help("The request ID to use when communicating with the Azure Functions Host.")
                        .hidden(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("max_message_length")
                        .long("grpcMaxMessageLength")
                        .value_name("MAXIMUM")
                        .help("The maximum message length to use for gRPC messages.")
                )
        )
}
