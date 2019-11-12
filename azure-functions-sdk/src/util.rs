use colored::Colorize;
use handlebars::Handlebars;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;

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

pub fn last_segment_in_path(path: &syn::Path) -> &syn::PathSegment {
    path.segments
        .iter()
        .last()
        .expect("expected at least one segment in path")
}
