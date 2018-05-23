#![feature(proc_macro)]

extern crate azure_functions;
#[macro_use]
extern crate log;

mod greet;

azure_functions::main!{
    greet::greet
}
