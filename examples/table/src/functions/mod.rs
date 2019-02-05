mod create_row;
mod read_row;

use azure_functions::{codegen::Function, export};

pub const FUNCTIONS: &[&Function] = export! {
    create_row::create_row,
    read_row::read_row,
};
