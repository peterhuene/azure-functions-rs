mod timer;

pub const FUNCTIONS: &[&azure_functions::codegen::Function] = azure_functions::export! {
    timer::timer,
};
