use colored::Colorize;
use handlebars::Handlebars;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use toml::Value;

pub fn print_running(message: &str) {
    print!(
        "{} {}",
        if cfg!(windows) { "->" } else { "️🚀" }.cyan(),
        message
    );
}

pub fn print_success() {
    println!(" {}", if cfg!(windows) { "OK" } else { "️✓" }.green());
}

pub fn print_failure() {
    println!(" {}", if cfg!(windows) { "FAIL" } else { "✗" }.red());
}

pub fn create_from_template(
    templates: &Handlebars,
    template_name: &str,
    base_path: &str,
    relative_path: &str,
    data: &serde_json::Value,
) -> Result<(), String> {
    let output_path = Path::new(base_path).join(relative_path);

    if let Some(dir) = output_path.parent() {
        create_dir_all(&dir)
            .unwrap_or_else(|_| panic!("failed to create directory for '{}'", relative_path));
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)
        .map_err(|e| format!("failed to create '{}': {}", relative_path.cyan(), e))?;

    file.write_all(
        templates
            .render(template_name, data)
            .map_err(|e| format!("failed to render '{}': {}", relative_path.cyan(), e))?
            .as_bytes(),
    )
    .map_err(|e| format!("failed to write {}: {}", relative_path.cyan(), e))?;

    Ok(())
}

pub fn path_to_string(path: &syn::Path) -> String {
    let mut s = String::new();

    for segment in path.segments.iter() {
        if !s.is_empty() {
            s += "::";
        }

        s += &segment.ident.to_string();
    }

    s
}

pub fn read_crate_name(path: &str) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|e| format!("Failed to open {}: {}", "Cargo.toml".cyan(), e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
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
