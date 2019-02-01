mod greet;
mod greet_with_json;

use azure_functions::{codegen::Function, export};

pub const FUNCTIONS: &[&Function] = export! {
    greet::greet,
    greet_with_json::greet_with_json,
};
