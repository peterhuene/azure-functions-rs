use atty::Stream;
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::Colorize;

use std::process::Command;

use crate::util::{print_failure, print_running, print_success, read_crate_name};

pub struct Run<'a> {
    quiet: bool,
    no_build: bool,
    color: Option<&'a str>,
    port: Option<&'a str>,
    image: Option<&'a str>,
}

impl<'a> Run<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("run")
            .about("Runs an Azure Functions application in a Docker container.")
            .arg(
                Arg::with_name("quiet")
                    .long("quiet")
                    .short("q")
                    .help("No output printed to stdout."),
            )
            .arg(
                Arg::with_name("no-build")
                    .long("no-build")
                    .help("Skips building the Azure Functions application prior to running."),
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
                Arg::with_name("image")
                    .value_name("IMAGE")
                    .help("The image of the Azure Function application to run. Default is based off the crate name.")
                    .index(1),
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

        let image = match self.image {
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

        let image = image
            .as_ref()
            .map_or_else(|| self.image.unwrap(), |img| img);

        if !self.no_build {
            crate::commands::Build::new(self.quiet, self.color, Some(image)).execute()?;
        }

        self.run_image(image)?;

        Ok(())
    }

    fn run_image(&self, image: &str) -> Result<(), String> {
        let port = format!("{}:80", self.port.unwrap_or("8080"));
        let args = &[
            "run",
            "-it",
            "-p",
            &port,
            "-e",
            "AzureWebJobsStorage",
            image,
        ];

        if !self.quiet {
            print_running(&format!(
                "spawning docker to run image: {}",
                format!("docker {}", args.join(" ")).cyan()
            ));
        }

        let mut child = Command::new("docker")
            .args(args)
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

impl<'a> From<&'a ArgMatches<'a>> for Run<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Run {
            quiet: args.is_present("quiet"),
            no_build: args.is_present("no-build"),
            color: args.value_of("color"),
            port: args.value_of("port"),
            image: args.value_of("image"),
        }
    }
}
