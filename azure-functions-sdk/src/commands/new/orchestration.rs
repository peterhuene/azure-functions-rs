use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct Orchestration<'a> {
    name: &'a str,
}

impl<'a> Orchestration<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("orchestration")
            .about("Creates a new Orchestration Function for Durable Functions.")
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
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
        });

        create_function(self.name, "orchestration.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Orchestration<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Orchestration {
            name: args
                .value_of("positional-name")
                .unwrap_or_else(|| args.value_of("name").expect("A NAME argument is needed")),
        }
    }
}
