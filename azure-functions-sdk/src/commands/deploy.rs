use crate::util::{print_failure, print_running, print_success, read_crate_name};
use clap::ArgMatches;
use colored::Colorize;

pub struct Deploy<'a> {
  verbose: bool,
  quiet: bool,
  color: Option<&'a str>,
  image: Option<&'a str>,
}

impl<'a> Deploy<'a> {
  pub fn execute(&self) -> Result<(), String> {
    super::set_colorization(self.color);

    let image = self.image()?;
    // Build image
    // run upload image
    Ok(())
  }

  fn image(&self) -> Result<String, String> {
    let image = match self.image {
      None => {
        if !self.quiet {
          print_running(&format!(
            "reading {} to determine default Docker image name",
            "Cargo.toml".cyan()
          ));
        }

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
          })?
      }
      Some(i) => i.into(),
    };

    Ok(image)
  }
}

impl<'a> From<&'a ArgMatches<'a>> for Deploy<'a> {
  fn from(args: &'a ArgMatches<'a>) -> Self {
    Deploy {
      verbose: args.is_present("verbose"),
      quiet: args.is_present("quiet"),
      color: args.value_of("color"),
      image: args.value_of("image"),
    }
  }
}
