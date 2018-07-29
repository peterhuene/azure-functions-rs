#![feature(use_extern_macros)]

extern crate azure_functions;
#[macro_use]
extern crate log;

mod blob;

azure_functions::main!{
    blob::print_blob,
}
