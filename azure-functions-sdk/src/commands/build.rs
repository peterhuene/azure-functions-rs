use crate::util::{print_failure, print_running, print_success, read_crate_name};
use atty::Stream;
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use std::process::Command;

pub struct Build<'a> {
    quiet: bool,
    color: Option<&'a str>,
    tag: Option<&'a str>,
}

impl<'a> Build<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("build")
            .about("Builds a Docker image for the Azure Functions application.")
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
                    .help("The tag to use for the image. Default is based off the crate name."),
            )
    }

    pub fn new(quiet: bool, color: Option<&'a str>, tag: Option<&'a str>) -> Build<'a> {
        Build { quiet, color, tag }
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
                        "reading {} to determine default Docker image name",
                        "Cargo.toml".cyan()
                    ));
                }
                Some(
                    read_crate_name("Cargo.toml")
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
}

impl<'a> From<&'a ArgMatches<'a>> for Build<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Build::new(
            args.is_present("quiet"),
            args.value_of("color"),
            args.value_of("tag"),
        )
    }
}
