use crate::{
    commands::TEMPLATES,
    util::{create_from_template, print_failure, print_running, print_success},
};
use atty::Stream;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use regex::Regex;
use serde_json::{json, Value};
use std::{
    fmt::Write,
    fs::{remove_file, File},
    io::Read,
    path::Path,
};
use syn::{parse_file, Expr, ExprArray, Item};

mod blob;
mod cosmos_db;
mod event_grid;
mod event_hub;
mod http;
mod queue;
mod timer;

pub use self::blob::Blob;
pub use self::cosmos_db::CosmosDb;
pub use self::event_grid::EventGrid;
pub use self::event_hub::EventHub;
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

fn format_path(path: &syn::Path) -> String {
    let mut formatted = String::new();
    if path.leading_colon.is_some() {
        formatted.push_str("::");
    }

    let mut first = true;
    for segment in &path.segments {
        if first {
            first = false;
        } else {
            formatted.push_str("::");
        }

        write!(formatted, "{}", segment.ident).unwrap();
    }

    formatted
}

fn parse_array_elements(array: &ExprArray, functions: &mut Vec<String>) -> bool {
    for elem in &array.elems {
        match elem {
            Expr::Reference(r) => match &*r.expr {
                Expr::Path(p) => {
                    functions.push(format_path(&p.path));
                }
                _ => {
                    return false;
                }
            },
            _ => {
                return false;
            }
        }
    }

    true
}

fn export_function(name: &str) -> Result<(), String> {
    let mut file =
        File::open("src/functions/mod.rs").map_err(|_| "'src/functions/mod.rs' does not exist.")?;

    let mut source = String::new();
    file.read_to_string(&mut source)
        .map_err(|_| "failed to read 'src/functions/mod.rs'.")?;

    let ast = parse_file(&source).map_err(|_| "failed to parse 'src/functions/mod.rs'.")?;

    let mut modules = Vec::new();
    let mut exports = None;

    for item in ast.items {
        match item {
            Item::Mod(m) => {
                modules.push(m.ident.to_string());
            }
            Item::Const(c) => {
                if exports.is_some() {
                    return Err(
                        "multiple 'EXPORTS' constants found in 'src/functions/mod.rs'.".to_string(),
                    );
                }

                if c.ident == "EXPORTS" {
                    exports = Some(c);
                }
            }
            _ => {}
        }
    }

    let mut functions = Vec::new();

    let found = match &exports {
        None => false,
        Some(exports) => match &*exports.expr {
            Expr::Reference(r) => match &*r.expr {
                Expr::Array(a) => parse_array_elements(a, &mut functions),
                _ => false,
            },
            _ => false,
        },
    };

    if !found {
        return Err("failed to find 'EXPORTS' constant in 'src/functions/mod.rs'.".to_string());
    }

    modules.push(name.to_string());
    modules.sort();

    functions.push(format!("{}::{}_FUNCTION", name, name.to_uppercase()));
    functions.sort();

    create_from_template(
        &TEMPLATES,
        "functions_mod.rs",
        "",
        "src/functions/mod.rs",
        &json!({
            "modules": modules,
            "functions": functions
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
