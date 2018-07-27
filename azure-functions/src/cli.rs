use clap::{App, Arg, SubCommand};

pub fn create_app() -> App<'a, 'b> {
    App::new("Azure Functions Language Worker for Rust")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Provides an Azure Functions Worker for functions written in Rust.")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initializes the Rust language worker and script root.")
                .arg(
                    Arg::with_name("worker_path")
                        .long("worker-path")
                        .value_name("WORKER_PATH")
                        .help("The path to place the Rust language worker.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("script_root")
                        .long("script-root")
                        .value_name("SCRIPT_ROOT")
                        .help("The directory to create the script root.")
                        .required(true),
                )
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs the Rust language worker.")
                 .arg(
                    Arg::with_name("worker_config")
                        .value_name("WORKER_CONFIG")
                        .help("The path to the Rust worker configuration file.")
                        .required(false)
                        .index(1)
                )
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
