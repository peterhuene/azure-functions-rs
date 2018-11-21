use atty::Stream;
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use toml::Value;
use {print_failure, print_running, print_success};

pub struct Build<'a> {
    quiet: bool,
    color: Option<&'a str>,
    tag: Option<&'a str>,
}

impl Build<'a> {
    pub fn create_subcommand() -> App<'a, 'b> {
        SubCommand::with_name("build")
            .about("Build a Docker image for the Azure Functions application.")
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
                Arg::with_name("tag")
                    .long("tag")
                    .short("t")
                    .value_name("TAG")
                    .help("The tag to use for the image."),
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

        self.build_image()?;

        Ok(())
    }

    fn build_image(&self) -> Result<(), String> {
        let tag = match self.tag {
            None => {
                if !self.quiet {
                    print_running(&format!(
                        "reading {} to determine default Docker image tag name",
                        "Cargo.toml".cyan()
                    ));
                }
                Some(
                    self.get_crate_name()
                        .map(|s| {
                            if !self.quiet {
                                print_success();
                            }
                            format!("azure-functions/{}:latest", s)
                        })
                        .map_err(|e| {
                            if !self.quiet {
                                print_failure();
                            }
                            e
                        })?,
                )
            }
            Some(_) => None,
        };

        let mut args = vec!["build", "--progress", "plain"];
        args.push("-t");
        args.push(tag.as_ref().map_or_else(|| self.tag.unwrap(), |t| t));
        args.push(".");

        if !self.quiet {
            print_running(&format!(
                "spawning docker to build image: {}",
                format!("docker {}", args.join(" ")).cyan()
            ));
        }

        let mut child = Command::new("docker")
            .args(args)
            .env("DOCKER_BUILDKIT", "1")
            .spawn()
            .map_err(|e| format!("failed to spawn docker: {}", e))?;

        if !self.quiet {
            print_success();
        }

        let status = child
            .wait()
            .map_err(|e| format!("failed to wait for docker: {}", e))?;

        if !status.success() {
            return Err(format!(
                "docker failed with exit code {}.",
                status.code().unwrap()
            ));
        }

        Ok(())
    }

    fn get_crate_name(&self) -> Result<String, String> {
        let mut _file = File::open("Cargo.toml")
            .map_err(|e| format!("Failed to open {}: {}", "Cargo.toml".cyan(), e))?;

        let mut contents = String::new();
        _file
            .read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", "Cargo.toml".cyan(), e))?;

        let value: Value = contents
            .as_str()
            .parse::<Value>()
            .map_err(|e| format!("Failed to decode {}: {}", "Cargo.toml".cyan(), e))?;

        let table = value.as_table().ok_or_else(|| {
            format!(
                "Expected a table for {} but found {}.",
                "Cargo.toml".cyan(),
                value.type_str()
            )
        })?;

        let package = table
            .get("package")
            .ok_or_else(|| {
                format!(
                    "{} does not contain a 'package' table.",
                    "Cargo.toml".cyan()
                )
            })?
            .as_table()
            .ok_or_else(|| {
                format!(
                    "{} contains a 'package' entry that is not a table.",
                    "Cargo.toml".cyan()
                )
            })?;

        let name = package
            .get("name")
            .ok_or_else(|| {
                format!(
                    "{} does not contain a 'package' table.",
                    "Cargo.toml".cyan()
                )
            })?
            .as_str()
            .ok_or_else(|| {
                format!(
                    "{} contains a 'package.name' entry that is not a string.",
                    "Cargo.toml".cyan()
                )
            })?;

        Ok(name.to_owned())
    }
}

impl From<&'a ArgMatches<'a>> for Build<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Build {
            quiet: args.is_present("quiet"),
            color: args.value_of("color"),
            tag: args.value_of("tag"),
        }
    }
}
