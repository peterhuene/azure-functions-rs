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

mod blob;
mod event_grid;
mod http;
mod queue;
mod timer;

pub use self::blob::Blob;
pub use self::event_grid::EventGrid;
pub use self::http::Http;
pub use self::queue::Queue;
pub use self::timer::Timer;

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

    let mut modules = Vec::new();
    let mut exports = Vec::new();

    for item in ast.items {
        match item {
            Item::Mod(m) => {
                modules.push(m.ident.to_string());
            }
            Item::Macro(m) => {
                if last_segment_in_path(&m.mac.path).ident == "export" {
                    exports.extend(
                        Punctuated::<Ident, Token![,]>::parse_terminated
                            .parse2(m.mac.tts)
                            .map_err(|_| "failed to parse 'export!' macro.")?
                            .into_iter()
                            .map(|i| i.to_string()),
                    );
                }
            }
            _ => {}
        }
    }

    modules.push(name.to_string());
    modules.sort();

    exports.push(name.to_string());
    exports.sort();

    create_from_template(
        &TEMPLATES,
        "functions_mod.rs",
        "",
        "src/functions/mod.rs",
        &json!({
            "modules": modules,
            "exports": exports
        }),
    )
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
            .subcommand(CosmosDb::create_subcommand())
            .subcommand(ServiceBus::create_subcommand())
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
            ("cosmos-db", Some(args)) => CosmosDb::from(args).execute(self.quiet),
            ("service-bus", Some(args)) => ServiceBus::from(args).execute(self.quiet),
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

struct CosmosDb<'a> {
    name: &'a str,
    connection: &'a str,
    database: &'a str,
    collection: &'a str,
}

impl<'a> CosmosDb<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("cosmos-db")
            .about("Creates a new Cosmos DB triggered Azure Function.")
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
                    .help("The name of the connection setting to use for the Cosmos DB trigger.")
                    .required(true),
            )
            .arg(
                Arg::with_name("database")
                    .long("database")
                    .short("d")
                    .value_name("DATABASE")
                    .help("The name of the database to use for the Cosmos DB trigger.")
                    .required(true),
            )
            .arg(
                Arg::with_name("collection")
                    .long("collection")
                    .short("l")
                    .value_name("COLLECTION")
                    .help("The name of the collection to use for the Cosmos DB trigger.")
                    .required(true),
            )
    }

    pub fn execute(&self, quiet: bool) -> Result<(), String> {
        let data = json!({
            "name": self.name,
            "connection": self.connection,
            "database": self.database,
            "collection": self.collection,
        });

        create_function(self.name, "cosmosdb.rs", &data, quiet)
    }
}

impl<'a> From<&'a ArgMatches<'a>> for CosmosDb<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        CosmosDb {
            name: args.value_of("name").unwrap(),
            connection: args.value_of("connection").unwrap(),
            database: args.value_of("database").unwrap(),
            collection: args.value_of("collection").unwrap(),
        }
    }
}

struct ServiceBus<'a> {
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
            name: args.value_of("name").unwrap(),
            connection: args.value_of("connection").unwrap(),
            queue: args.value_of("queue"),
            topic: args.value_of("topic"),
            subscription: args.value_of("subscription"),
        }
    }
}
