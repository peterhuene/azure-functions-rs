use crate::{
    commands::TEMPLATES,
    util::{
        create_from_template, last_segment_in_path, print_failure, print_running, print_success,
    },
};
use atty::Stream;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use regex::Regex;
use serde_json::{json, Value};
use std::{
    fs::{remove_file, File},
    io::Read,
    path::Path,
};
use syn::{parse::Parser, parse_file, punctuated::Punctuated, Ident, Item, Token};

fn get_path_for_function(name: &str) -> Result<String, String> {
    if !Regex::new("^[a-zA-Z][a-zA-Z0-9_]*$")
        .unwrap()
        .is_match(name)
    {
        return Err("Function name must start with a letter and only contain letters, numbers, and underscores.".to_string());
    }

    if name.len() > 127 {
        return Err("Function names cannot exceed 127 characters.".to_string());
    }

    if !Path::new("src/functions").is_dir() {
        return Err("Directory 'src/functions' does not exist.".to_string());
    }

    let path = format!("src/functions/{}.rs", name);
    if Path::new(&path).exists() {
        return Err(format!("'{}' already exists.", path));
    }

    Ok(path)
}

fn create_function(name: &str, template: &str, data: &Value, quiet: bool) -> Result<(), String> {
    let path = get_path_for_function(name)?;

    if !quiet {
        print_running(&format!("creating {}.", path.cyan()));
    }

    create_from_template(&TEMPLATES, template, "", &path, data)
        .map(|_| {
            if !quiet {
                print_success();
            }
        })
        .map_err(|e| {
            if !quiet {
                print_failure();
            }
            e
        })?;

    if !quiet {
        print_running(&format!(
            "exporting function {} in {}.",
            name.cyan(),
            "src/functions/mod.rs".cyan()
        ));
    }

    export_function(name)
        .map(|_| {
            if !quiet {
                print_success();
            }
        })
        .map_err(|e| {
            if !quiet {
                print_failure();
            }
            remove_file(path).expect("failed to delete source file");
            e
        })?;

    Ok(())
}

fn export_function(name: &str) -> Result<(), String> {
    let mut file =
        File::open("src/functions/mod.rs").map_err(|_| "'src/functions/mod.rs' does not exist.")?;

    let mut source = String::new();
    file.read_to_string(&mut source)
        .map_err(|_| "failed to read 'src/functions/mod.rs'.")?;

    let ast = parse_file(&source).map_err(|_| "failed to parse 'src/functions/mod.rs'.")?;

    let mut found = false;
    for item in ast.items {
        if let Item::Macro(m) = item {
            if last_segment_in_path(&m.mac.path).ident == "export" {
                let mut modules: Vec<String> = Punctuated::<Ident, Token![,]>::parse_terminated
                    .parse2(m.mac.tts)
                    .map_err(|_| "failed to parse 'azure_functions::export' macro.")?
                    .into_iter()
                    .map(|i| i.to_string())
                    .collect();

                modules.push(name.to_string());

                modules.sort();

                create_from_template(
                    &TEMPLATES,
                    "functions_mod.rs",
                    "",
                    "src/functions/mod.rs",
                    &json!({ "modules": if modules.is_empty() {
                        String::new()
                    } else {
                        format!("    {}", modules.join(",\n    "))
                    }}),
                )?;

                found = true;
            }
        }
    }

    if !found {
        return Err("failed to find 'export!' macro in 'src/functions/mod.rs'.".to_string());
    }

    Ok(())
}

pub struct New<'a> {
    quiet: bool,
    color: Option<&'a str>,
    args: &'a ArgMatches<'a>,
}

impl<'a> New<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("new")
            .about("Creates a new Azure Function from a template.")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .arg(
                Arg::with_name("quiet")
                    .long("quiet")
                    .short("q")
                    .help("No output printed to stdout."),
            )
            .arg(
                Arg::with_name("color")
                    .long("color")
                    .value_name("WHEN")
                    .help("Controls when colored output is enabled.")
                    .possible_values(&["auto", "always", "never"])
                    .default_value("auto"),
            )
            .subcommand(Blob::create_subcommand())
            .subcommand(Http::create_subcommand())
            .subcommand(Queue::create_subcommand())
            .subcommand(Timer::create_subcommand())
            .subcommand(EventGrid::create_subcommand())
            .subcommand(EventHub::create_subcommand())
    }

    fn set_colorization(&self) {
        ::colored::control::set_override(match self.color {
            Some("auto") | None => ::atty::is(Stream::Stdout),
            Some("always") => true,
            Some("never") => false,
            _ => panic!("unsupported color option"),
        });
    }

    pub fn execute(&self) -> Result<(), String> {
        self.set_colorization();

        match self.args.subcommand() {
            ("blob", Some(args)) => Blob::from(args).execute(self.quiet),
            ("http", Some(args)) => Http::from(args).execute(self.quiet),
            ("queue", Some(args)) => Queue::from(args).execute(self.quiet),
            ("timer", Some(args)) => Timer::from(args).execute(self.quiet),
            ("event-grid", Some(args)) => EventGrid::from(args).execute(self.quiet),
            ("event-hub", Some(args)) => EventHub::from(args).execute(self.quiet),
            _ => panic!("expected a subcommand for the 'new' command."),
        }
    }
}

impl<'a> From<&'a ArgMatches<'a>> for New<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        New {
            quiet: args.is_present("quiet"),
            color: args.value_of("color"),
            args,
        }
    }
}

struct Http<'a> {
    name: &'a str,
    auth_level: &'a str,
}

impl<'a> Http<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("http")
            .about("Creates a new HTTP triggered Azure Function.")
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function.")
                    .required(true),
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
            name: args.value_of("name").unwrap(),
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

struct Blob<'a> {
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

struct Queue<'a> {
    name: &'a str,
    queue_name: &'a str,
}

impl<'a> Queue<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("queue")
            .about("Creates a new queue triggered Azure Function.")
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function.")
                    .required(true),
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
            name: args.value_of("name").unwrap(),
            queue_name: args.value_of("queue_name").unwrap(),
        }
    }
}

struct Timer<'a> {
    name: &'a str,
    schedule: &'a str,
}

impl<'a> Timer<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("timer")
            .about("Creates a new timer triggered Azure Function.")
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function.")
                    .required(true),
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
            name: args.value_of("name").unwrap(),
            schedule: args.value_of("schedule").unwrap(),
        }
    }
}

struct EventGrid<'a> {
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

struct EventHub<'a> {
    name: &'a str,
    connection: &'a str,
    hub_name: &'a str,
}

impl<'a> EventHub<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("event-hub")
            .about("Creates a new Event Hub triggered Azure Function.")
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("The name of the new Azure Function.")
                    .required(true),
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
            name: args.value_of("name").unwrap(),
            connection: args.value_of("connection").unwrap(),
            hub_name: args.value_of("hub_name").unwrap_or(""),
        }
    }
}
