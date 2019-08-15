use crate::commands::new::create_function;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::json;

pub struct CosmosDb<'a> {
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
            name: args.value_of("positional-name")
                    .unwrap_or_else(|| args.value_of("name")
                    .unwrap_or("Default fallback - never reached")),
            connection: args.value_of("connection").unwrap(),
            database: args.value_of("database").unwrap(),
            collection: args.value_of("collection").unwrap(),
        }
    }
}
