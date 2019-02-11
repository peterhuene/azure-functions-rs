use atty::Stream;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, Values};
use colored::Colorize;
use std::env::current_dir;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

use crate::util::{print_failure, print_running, print_success};

pub struct Run<'a> {
    quiet: bool,
    color: Option<&'a str>,
    port: Option<&'a str>,
    script_root: Option<&'a str>,
    cargo_options: Option<Values<'a>>,
}

impl<'a> Run<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("run")
            .setting(AppSettings::TrailingVarArg)
            .usage("cargo func run [FLAGS] [OPTIONS] -- [CARGO_OPTIONS]...")
            .about("Runs an Azure Functions application using a local Azure Functions Host.")
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
            .arg(
                Arg::with_name("port")
                    .long("port")
                    .short("p")
                    .value_name("PORT")
                    .help("The port to forward to the Azure Functions Host for HTTP requests. Default is 8080."),
            )
            .arg(
                Arg::with_name("script_root")
                    .long("script-root")
                    .short("r")
                    .value_name("ROOT")
                    .help("The directory to use for the Azure Functions application script root. Default is a temporary directory."),
            )
            .arg(Arg::with_name("cargo_options")
                .multiple(true)
                .value_name("CARGO_OPTIONS")
                .help("Additional options to pass to 'cargo run'."),
            )
    }

    fn set_colorization(&self) {
        colored::control::set_override(match self.color {
            Some("auto") | None => atty::is(Stream::Stdout),
            Some("always") => true,
            Some("never") => false,
            _ => panic!("unsupported color option"),
        });
    }

    pub fn execute(&self) -> Result<(), String> {
        self.set_colorization();

        let (_temp_dir, script_root) = match self.script_root {
            Some(dir) => {
                let script_root = current_dir()
                    .expect("failed to get current directory")
                    .join(dir);
                (None, script_root)
            }
            None => {
                let temp_dir = TempDir::new("script-root")
                    .map_err(|e| format!("failed to create temp directory: {}", e))?;
                let script_root = temp_dir.path().to_owned();
                (Some(temp_dir), script_root)
            }
        };

        self.init_script_root(&script_root)?;

        self.run_host(&script_root)?;

        Ok(())
    }

    fn init_script_root(&self, script_root: &Path) -> Result<(), String> {
        let mut args = vec!["run"];

        match self.cargo_options.as_ref() {
            Some(values) => {
                for value in values.clone() {
                    args.push(value);
                }
            }
            _ => {}
        };

        args.extend_from_slice(&[
            "--",
            "init",
            "--script-root",
            script_root.to_str().unwrap(),
            "--sync",
        ]);

        if !self.quiet {
            print_running(&format!(
                "spawning 'cargo' to initialize script root: {}",
                format!("cargo {}", args.join(" ")).cyan()
            ));
        }

        let mut child = Command::new("cargo").args(&args).spawn().map_err(|e| {
            if !self.quiet {
                print_failure();
            }
            format!("failed to spawn cargo: {}", e)
        })?;

        if !self.quiet {
            print_success();
        }

        let status = child
            .wait()
            .map_err(|e| format!("failed to wait for cargo: {}", e))?;

        if !status.success() {
            return Err(format!(
                "cargo failed with exit code {}.",
                status.code().unwrap()
            ));
        }

        Ok(())
    }

    fn run_host(&self, script_root: &Path) -> Result<(), String> {
        let args = ["host", "start", "--port", self.port.unwrap_or("8080")];

        if !self.quiet {
            print_running(&format!(
                "spawning 'func' to start the Azure Functions Host: {}",
                format!("func {}", args.join(" ")).cyan()
            ));
        }

        let mut child = Command::new("func")
            .args(&args)
            .current_dir(script_root)
            .spawn()
            .map_err(|e| {
                if !self.quiet {
                    print_failure();
                }
                format!("failed to spawn func: {}\nensure the Azure Functions Core Tools are installed.", e)
            })?;

        if !self.quiet {
            print_success();
        }

        let status = child
            .wait()
            .map_err(|e| format!("failed to wait for func: {}", e))?;

        if !status.success() {
            return Err(format!(
                "func failed with exit code {}.",
                status.code().unwrap()
            ));
        }

        Ok(())
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Run<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Run {
            quiet: args.is_present("quiet"),
            color: args.value_of("color"),
            port: args.value_of("port"),
            script_root: args.value_of("script_root"),
            cargo_options: args.values_of("cargo_options"),
        }
    }
}
