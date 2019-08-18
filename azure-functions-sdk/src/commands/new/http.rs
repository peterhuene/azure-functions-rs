use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct Http<'a> {
    name: &'a str,
    auth_level: &'a str,
}

impl<'a> Http<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("http")
            .about("Creates a new HTTP triggered Azure Function.")
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
                Arg::with_name("auth-level")
                    .long("auth-level")
                    .value_name("LEVEL")
                    .possible_values(&["anonymous", "function", "admin"])
                    .help("The authentication level for the HTTP function. Default is 'function'."),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
            "auth_level": self.auth_level
        });

        create_function(self.name, "http.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Http<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Http {
            name: args.value_of("positional-name").unwrap_or_else(|| {
                args.value_of("name")
                    .unwrap_or("Default fallback - never reached")
            }),
            auth_level: match args.value_of("auth-level") {
                Some(level) => {
                    if level == "function" {
                        ""
                    } else {
                        level
                    }
                }
                None => "",
            },
        }
    }
}
