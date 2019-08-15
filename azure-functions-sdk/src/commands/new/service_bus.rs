use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct ServiceBus<'a> {
    name: &'a str,
    connection: &'a str,
    queue: Option<&'a str>,
    topic: Option<&'a str>,
    subscription: Option<&'a str>,
}

impl<'a> ServiceBus<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("service-bus")
            .about("Creates a new Service Bus triggered Azure Function.")
            .arg(
                Arg::with_name("positional-name")
                    .value_name("NAME")
                    .help("The name of the new Azure Function. You may specify this as --name <NAME> instead.")
                    .conflicts_with("name")
                    .required(true),
            )
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function. You may specify this as <NAME> instead (i.e., without typing --name).")
            )
            .arg(
                Arg::with_name("connection")
                    .long("connection")
                    .short("c")
                    .value_name("CONNECTION")
                    .help("The name of the connection setting to use for the Service Bus trigger.")
                    .required(true),
            )
            .arg(
                Arg::with_name("queue")
                    .long("queue")
                    .short("q")
                    .value_name("QUEUE")
                    .help("The name of the queue to use for the Service Bus trigger.")
                    .conflicts_with_all(&["topic", "subscription"])
                    .required(true),
            )
            .arg(
                Arg::with_name("topic")
                    .long("topic")
                    .short("t")
                    .value_name("TOPIC")
                    .help("The name of the topic to use for the Service Bus trigger.")
                    .conflicts_with("queue")
                    .required(true),
            )
            .arg(
                Arg::with_name("subscription")
                    .long("subscription")
                    .short("s")
                    .value_name("SUBSCRIPTION")
                    .help("The name of the subscription to use for the Service Bus trigger.")
                    .conflicts_with("queue")
                    .required(true),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
            "connection": self.connection,
            "queue": self.queue,
            "topic": self.topic,
            "subscription": self.subscription
        });

        create_function(self.name, "servicebus.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for ServiceBus<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        ServiceBus {
            name: args.value_of("positional-name")
                    .unwrap_or_else(|| args.value_of("name")
                    .unwrap_or("Default fallback - never reached")),
            connection: args.value_of("connection").unwrap(),
            queue: args.value_of("queue"),
            topic: args.value_of("topic"),
            subscription: args.value_of("subscription"),
        }
    }
}
