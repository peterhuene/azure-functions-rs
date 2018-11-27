mod greet;
mod greet_with_json;

pub const FUNCTIONS: &[&azure_functions::codegen::Function] = azure_functions::export! {
    greet::greet,
    greet_with_json::greet_with_json,
};
