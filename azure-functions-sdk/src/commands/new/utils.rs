use crate::{
    util::{create_from_template, print_failure, print_running, print_success},
};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    fmt::Write,
    fs::{remove_file, File},
    io::Read,
    path::Path,
};
use syn::{parse_file, Expr, ExprArray, Item};

fn get_path_for_function(name: &str) -> Result<String, String> {
    if !Regex::new("^[a-zA-Z][a-zA-Z0-9_]*$")
        .unwrap()
        .is_match(name)
    {
        return Err("Function name must start with a letter and only contain letters, numbers, and underscores.".to_string());
    }

    if name.len() > 127 {
        return Err("Function names cannot exceed 127 characters.".to_string());
    }

    if !Path::new("src/functions").is_dir() {
        return Err("Directory 'src/functions' does not exist.".to_string());
    }

    let path = format!("src/functions/{}.rs", name);
    if Path::new(&path).exists() {
        return Err(format!("'{}' already exists.", path));
    }

    Ok(path)
}

fn create_function(name: &str, template: &str, data: &Value, quiet: bool) -> Result<(), String> {
    let path = get_path_for_function(name)?;

    if !quiet {
        print_running(&format!("creating {}.", path.cyan()));
    }

    create_from_template(&TEMPLATES, template, "", &path, data)
        .map(|_| {
            if !quiet {
                print_success();
            }
        })
        .map_err(|e| {
            if !quiet {
                print_failure();
            }
            e
        })?;

    if !quiet {
        print_running(&format!(
            "exporting function {} in {}.",
            name.cyan(),
            "src/functions/mod.rs".cyan()
        ));
    }

    export_function(name)
        .map(|_| {
            if !quiet {
                print_success();
            }
        })
        .map_err(|e| {
            if !quiet {
                print_failure();
            }
            remove_file(path).expect("failed to delete source file");
            e
        })?;

    Ok(())
}

fn format_path(path: &syn::Path) -> String {
    let mut formatted = String::new();
    if path.leading_colon.is_some() {
        formatted.push_str("::");
    }

    let mut first = true;
    for segment in &path.segments {
        if first {
            first = false;
        } else {
            formatted.push_str("::");
        }

        write!(formatted, "{}", segment.ident).unwrap();
    }

    formatted
}

fn parse_array_elements(array: &ExprArray, functions: &mut Vec<String>) -> bool {
    for elem in &array.elems {
        match elem {
            Expr::Reference(r) => match &*r.expr {
                Expr::Path(p) => {
                    functions.push(format_path(&p.path));
                }
                _ => {
                    return false;
                }
            },
            _ => {
                return false;
            }
        }
    }

    true
}

fn export_function(name: &str) -> Result<(), String> {
    let mut file =
        File::open("src/functions/mod.rs").map_err(|_| "'src/functions/mod.rs' does not exist.")?;

    let mut source = String::new();
    file.read_to_string(&mut source)
        .map_err(|_| "failed to read 'src/functions/mod.rs'.")?;

    let ast = parse_file(&source).map_err(|_| "failed to parse 'src/functions/mod.rs'.")?;

    let mut modules = Vec::new();
    let mut exports = None;

    for item in ast.items {
        match item {
            Item::Mod(m) => {
                modules.push(m.ident.to_string());
            }
            Item::Const(c) => {
                if exports.is_some() {
                    return Err(
                        "multiple 'EXPORTS' constants found in 'src/functions/mod.rs'.".to_string(),
                    );
                }

                if c.ident == "EXPORTS" {
                    exports = Some(c);
                }
            }
            _ => {}
        }
    }

    let mut functions = Vec::new();

    let found = match &exports {
        None => false,
        Some(exports) => match &*exports.expr {
            Expr::Reference(r) => match &*r.expr {
                Expr::Array(a) => parse_array_elements(a, &mut functions),
                _ => false,
            },
            _ => false,
        },
    };

    if !found {
        return Err("failed to find 'EXPORTS' constant in 'src/functions/mod.rs'.".to_string());
    }

    modules.push(name.to_string());
    modules.sort();

    functions.push(format!("{}::{}_FUNCTION", name, name.to_uppercase()));
    functions.sort();

    create_from_template(
        &TEMPLATES,
        "functions_mod.rs",
        "",
        "src/functions/mod.rs",
        &json!({
            "modules": modules,
            "functions": functions
        }),
    )
}