use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct Timer<'a> {
    name: &'a str,
    schedule: &'a str,
}

impl<'a> Timer<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("timer")
            .about("Creates a new timer triggered Azure Function.")
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
                Arg::with_name("schedule")
                    .long("schedule")
                    .short("s")
                    .value_name("SCHEDULE")
                    .help("The timer schedule as a cron-expression.")
                    .required(true),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
            "schedule": self.schedule,
        });

        create_function(self.name, "timer.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Timer<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Timer {
            name: args.value_of("positional-name").unwrap_or_else(|| {
                args.value_of("name")
                    .unwrap_or("Default fallback - never reached")
            }),
            schedule: args.value_of("schedule").unwrap(),
        }
    }
}
