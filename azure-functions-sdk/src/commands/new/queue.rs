use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use regex::Regex;
use serde_json::json;

pub struct Queue<'a> {
    name: &'a str,
    queue_name: &'a str,
}

impl<'a> Queue<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("queue")
            .about("Creates a new queue triggered Azure Function.")
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
                Arg::with_name("queue_name")
                    .long("queue-name")
                    .short("q")
                    .value_name("QUEUE")
                    .help("The name of the storage queue to monitor.")
                    .required(true),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        Queue::validate_queue_name(self.queue_name)?;

        let data = json!({
            "name": self.name,
            "queue_name": self.queue_name
        });

        create_function(self.name, "queue.rs", &data, quiet)
    }

    fn validate_queue_name(name: &str) -> Result<(), String> {
        if name.len() < 3 {
            return Err(format!(
                "queue name '{}' must be at least 3 characters.",
                name
            ));
        }

        if name.len() > 63 {
            return Err(format!(
                "queue name '{}' cannot be more than 63 characters.",
                name
            ));
        }

        if !Regex::new("^[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]$")
            .unwrap()
            .is_match(name)
        {
            return Err(format!("queue name '{}' must start and end with a letter or number and can only contain letters, numbers, and hyphens.", name));
        }

        Ok(())
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Queue<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Queue {
            name: args.value_of("positional-name").unwrap_or_else(|| {
                args.value_of("name")
                    .expect("A NAME argument is needed")
            }),
            queue_name: args.value_of("queue_name").unwrap(),
        }
    }
}
