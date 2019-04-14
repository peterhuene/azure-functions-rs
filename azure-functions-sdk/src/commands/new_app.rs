use crate::{
    commands::TEMPLATES,
    util::{create_from_template, print_failure, print_running, print_success},
};
use atty::Stream;
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use serde_json::json;

use std::{
    fs::OpenOptions,
    io::{stdout, Write},
    path::Path,
    process::Command,
};

pub struct NewApp<'a> {
    path: &'a str,
    vcs: Option<&'a str>,
    name: Option<&'a str>,
    verbose: bool,
    quiet: bool,
    no_launch_config: bool,
    color: Option<&'a str>,
}

impl<'a> NewApp<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("new-app")
            .about("Creates a new Azure Functions application at the specified path.")
            .arg(
                Arg::with_name("vcs")
                    .long("vcs")
                    .value_name("VCS")
                    .help(
                        "Initialize a new repository for the given version control system. \
                         See `cargo new --help` for more information.",
                    )
                    .possible_values(&["git", "hg", "pijul", "fossil", "none"]),
            )
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .value_name("NAME")
                    .help("Set the resulting package name, defaults to the directory name."),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .help("Use verbose output."),
            )
            .arg(
                Arg::with_name("quiet")
                    .long("quiet")
                    .short("q")
                    .help("No output printed to stdout.")
                    .conflicts_with("verbose"),
            )
            .arg(
                Arg::with_name("no_launch_config")
                    .long("no-launch-config")
                    .help("Do not create a Visual Studio Code launch configuration."),
            )
            .arg(
                Arg::with_name("color")
                    .long("color")
                    .value_name("WHEN")
                    .help("Controls when colored output is enabled.")
                    .possible_values(&["auto", "always", "never"])
                    .default_value("auto"),
            )
            .arg(
                Arg::with_name("path")
                    .value_name("PATH")
                    .help("The path where the application should be created.")
                    .index(1)
                    .required(true),
            )
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

        self.create_crate()?;

        self.add_dependencies()
            .map(|_| {
                if !self.quiet {
                    print_success();
                }
            })
            .map_err(|e| {
                if !self.quiet {
                    print_failure();
                }
                e
            })?;

        let mut templates = vec![
            ("main.rs", "src/main.rs", json!({})),
            ("functions_mod.rs", "src/functions/mod.rs", json!({})),
            ("dockerignore", ".dockerignore", json!({})),
            (
                "Dockerfile",
                "Dockerfile",
                json!({ "crate_version": env!("CARGO_PKG_VERSION") }),
            ),
            ("appsettings.json", "appsettings.json", json!({})),
            ("local.settings.json", "local.settings.json", json!({})),
        ];

        if !self.no_launch_config {
            let name = self
                .name
                .unwrap_or_else(|| Path::new(self.path).file_name().unwrap().to_str().unwrap());

            templates.push((
                "launch.json",
                ".vscode/launch.json",
                json!({ "name": name }),
            ));
            templates.push(("tasks.json", ".vscode/tasks.json", json!({})));
        }

        for (template, relative_path, data) in templates.into_iter() {
            if !self.quiet {
                print_running(&format!("creating {}.", relative_path.cyan()));
            }

            create_from_template(&TEMPLATES, template, self.path, relative_path, &data)
                .map(|_| {
                    if !self.quiet {
                        print_success();
                    }
                })
                .map_err(|e| {
                    if !self.quiet {
                        print_failure();
                    }
                    e
                })?;
        }

        if !self.quiet {
            println!();
            println!(
                "{} Azure Functions application at {}.",
                "Created".green().bold(),
                self.path.cyan()
            );
        }

        Ok(())
    }

    fn create_crate(&self) -> Result<(), String> {
        let mut cargo_args = self.get_cargo_args();

        if !self.quiet {
            print_running(&format!(
                "running cargo to create an application: {}",
                format!("cargo {}", cargo_args.join(" ")).cyan()
            ));
        }

        // Silently treat auto with a TTY as always so that cargo will use color output
        if let Some(color) = self.color {
            if color == "auto" && ::atty::is(Stream::Stdout) {
                cargo_args.push("--color");
                cargo_args.push("always");
            }
        }

        let output = Command::new("cargo")
            .args(cargo_args)
            .output()
            .map_err(|e| format!("failed to spawn cargo: {}", e))?;

        if !self.quiet {
            if output.status.success() {
                print_success();
            } else {
                print_failure();
            }
        }

        if !output.stderr.is_empty() && (!output.status.success() || !self.quiet) {
            let stdout = stdout();
            stdout
                .lock()
                .write_all(&output.stderr)
                .map_err(|e| format!("failed to write cargo output: {}", e))?;
        }

        if !output.status.success() {
            return Err(format!(
                "cargo failed with exit code {}.",
                output.status.code().unwrap()
            ));
        }

        Ok(())
    }

    fn get_cargo_args(&self) -> Vec<&'a str> {
        let mut args = vec!["new", "--bin"];

        if let Some(vcs) = self.vcs {
            args.push("--vcs");
            args.push(vcs);
        }

        if let Some(name) = self.name {
            args.push("--name");
            args.push(name);
        }

        if self.verbose {
            args.push("--verbose");
        }

        if self.quiet {
            args.push("--quiet");
        }

        if let Some(color) = self.color {
            if color != "auto" {
                args.push("--color");
                args.push(color);
            }
        }

        args.push(self.path);

        args
    }

    fn add_dependencies(&self) -> Result<(), String> {
        if !self.quiet {
            print_running("adding dependencies for Azure Functions for Rust.");
        }

        let mut file = OpenOptions::new()
            .append(true)
            .open(Path::new(self.path).join("Cargo.toml"))
            .map_err(|e| format!("failed to open Cargo manifest: {}", e))?;

        file.write_all(
            concat!("azure-functions = \"", env!("CARGO_PKG_VERSION"), "\"\n").as_bytes(),
        )
        .map_err(|e| format!("failed to write dependencies to Cargo manifest: {}", e))?;

        Ok(())
    }
}

impl<'a> From<&'a ArgMatches<'a>> for NewApp<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        NewApp {
            path: args.value_of("path").unwrap(),
            vcs: args.value_of("vcs"),
            name: args.value_of("name"),
            verbose: args.is_present("verbose"),
            quiet: args.is_present("quiet"),
            no_launch_config: args.is_present("no_launch_config"),
            color: args.value_of("color"),
        }
    }
}
