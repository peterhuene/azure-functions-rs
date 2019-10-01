use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct EventHub<'a> {
    name: &'a str,
    connection: &'a str,
    hub_name: &'a str,
}

impl<'a> EventHub<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("event-hub")
            .about("Creates a new Event Hub triggered Azure Function.")
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
                    .help("The name of the connection setting to use for the Event Hub trigger.")
                    .required(true),
            )
            .arg(
                Arg::with_name("hub_name")
                    .long("hub-name")
                    .short("h")
                    .value_name("HUBNAME")
                    .help("The name of the Event Hub."),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
            "connection": self.connection,
            "hub_name": self.hub_name,
        });

        create_function(self.name, "eventhub.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for EventHub<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        EventHub {
            name: args.value_of("positional-name").unwrap_or_else(|| {
                args.value_of("name")
                    .expect("A NAME argument is needed")
            }),
            connection: args.value_of("connection").unwrap(),
            hub_name: args.value_of("hub_name").unwrap_or(""),
        }
    }
}
