#![feature(rust_2018_preview)]
#![feature(in_band_lifetimes)]
//#![deny(missing_docs)]
#![deny(unused_extern_crates)]

extern crate atty;
extern crate clap;
extern crate colored;
extern crate handlebars;
#[macro_use]
extern crate serde_json;
extern crate toml;

mod commands;
mod util;

use clap::{App, AppSettings};
use colored::Colorize;
use commands::{Build, NewApp, Run};
use std::env;
use std::process;

fn create_app() -> App<'a, 'b> {
    App::new("Azure Functions for Rust")
        .bin_name("cargo func")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Azure Functions for Rust Developer Tools")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::NoBinaryName)
        .subcommand(Build::create_subcommand())
        .subcommand(NewApp::create_subcommand())
        .subcommand(Run::create_subcommand())
}

fn print_error_and_exit(message: &str) {
    eprintln!("{}: {}", "error".red().bold(), message);
    process::exit(1);
}

fn main() {
    // Support both cargo-func <command> and cargo-func func <command>
    // This enables running both `cargo-func` and `cargo func`, which passes the `func` command down
    let mut matches = None;
    if let Some(first) = env::args().nth(1) {
        if first == "func" {
            matches = Some(create_app().get_matches_from(env::args().skip(2)));
        }
    }

    if let Err(e) = match matches
        .get_or_insert_with(|| create_app().get_matches_from(env::args().skip(1)))
        .subcommand()
    {
        ("new-app", Some(args)) => NewApp::from(args).execute(),
        ("build", Some(args)) => Build::from(args).execute(),
        ("run", Some(args)) => Run::from(args).execute(),
        _ => panic!("expected a subcommand."),
    } {
        print_error_and_exit(&e);
    }
}
