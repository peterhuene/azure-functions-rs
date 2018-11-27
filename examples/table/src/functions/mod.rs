mod create_row;
mod read_row;

pub const FUNCTIONS: &[&azure_functions::codegen::Function] = azure_functions::export! {
    create_row::create_row,
    read_row::read_row,
};
