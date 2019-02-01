mod timer;

use azure_functions::{codegen::Function, export};

pub const FUNCTIONS: &[&Function] = export! {
    timer::timer,
};
