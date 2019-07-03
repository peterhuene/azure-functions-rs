use crate::{registry::Registry, worker::Worker};
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct Run<'a> {
    pub host: &'a str,
    pub port: u16,
    pub worker_id: &'a str,
}

impl<'a> Run<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
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
                    .help("The maximum message length to use for gRPC messages."),
            )
    }

    pub fn execute(&self, registry: Registry<'static>) -> Result<(), String> {
        ctrlc::set_handler(|| {}).expect("failed setting SIGINT handler");

        Worker::run(self.host, self.port, self.worker_id, registry);

        Ok(())
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Run<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Run {
            host: args.value_of("host").expect("A host is required."),
            port: args
                .value_of("port")
                .map(|port| port.parse::<u16>().expect("Invalid port number"))
                .expect("A port number is required."),
            worker_id: args
                .value_of("worker_id")
                .expect("A worker id is required."),
        }
    }
}
