mod queue;
mod queue_with_output;

pub const FUNCTIONS: &[&azure_functions::codegen::Function] = azure_functions::export! {
    queue::queue,
    queue_with_output::queue_with_output,
};
