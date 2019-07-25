use crate::{
    commands::new::create_function,
};
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct Blob<'a> {
    name: &'a str,
    path: &'a str,
}

impl<'a> Blob<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("blob")
            .about("Creates a new blob triggered Azure Function.")
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function.")
                    .required(true),
            )
            .arg(
                Arg::with_name("path")
                    .long("path")
                    .short("p")
                    .value_name("PATH")
                    .help("The blob storage path to monitor.")
                    .required(true),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
            "path": self.path
        });

        create_function(self.name, "blob.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Blob<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Blob {
            name: args.value_of("name").unwrap(),
            path: args.value_of("path").unwrap(),
        }
    }
}