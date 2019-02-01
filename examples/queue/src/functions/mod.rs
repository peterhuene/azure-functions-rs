mod queue;
mod queue_with_output;

use azure_functions::{codegen::Function, export};

pub const FUNCTIONS: &[&Function] = export! {
    queue::queue,
    queue_with_output::queue_with_output,
};
