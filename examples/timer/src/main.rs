#![feature(proc_macro)]

extern crate azure_functions;
#[macro_use]
extern crate log;

mod timer;

azure_functions::main!{
    timer::timer,
}
