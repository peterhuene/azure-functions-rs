#![feature(use_extern_macros)]

extern crate azure_functions;
#[macro_use]
extern crate log;

mod timer;

azure_functions::main!{
    timer::timer,
}
