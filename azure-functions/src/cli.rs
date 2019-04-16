use crate::commands::{Init, Run};
use clap::{App, AppSettings, Arg, SubCommand};

pub fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Azure Functions for Rust worker")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Implements the Azure Functions for Rust worker.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(Init::create_subcommand())
        .subcommand(
            SubCommand::with_name("sync-extensions")
                .about("Synchronizes the Azure Function binding extensions used by the worker.")
                .arg(
                    Arg::with_name("script_root")
                        .long("script-root")
                        .value_name("SCRIPT_ROOT")
                        .help("The script root to synchronize the binding extensions for.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Use verbose output."),
                ),
        )
        .subcommand(Run::create_subcommand())
}
