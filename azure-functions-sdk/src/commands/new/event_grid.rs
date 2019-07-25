use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct EventGrid<'a> {
    name: &'a str,
}

impl<'a> EventGrid<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("event-grid")
            .about("Creates a new Event Grid triggered Azure Function.")
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function.")
                    .required(true),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
        });

        create_function(self.name, "eventgrid.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for EventGrid<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        EventGrid {
            name: args.value_of("name").unwrap(),
        }
    }
}
