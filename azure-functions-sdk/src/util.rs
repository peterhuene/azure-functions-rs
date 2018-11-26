use colored::Colorize;
use std::fs::File;
use std::io::Read;
use toml::Value;

pub fn print_running(message: &str) {
    print!("{} {}", "️🚀".cyan(), message);
}

pub fn print_success() {
    println!(" {}", "✓".green());
}

pub fn print_failure() {
    println!(" {}", "✗".red());
}

pub fn read_crate_name(path: &str) -> Result<String, String> {
    let mut _file =
        File::open(path).map_err(|e| format!("Failed to open {}: {}", "Cargo.toml".cyan(), e))?;

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
