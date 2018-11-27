#![feature(proc_macro_hygiene)]

extern crate azure_functions;
#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod functions;

pub fn main() {
    azure_functions::worker_main(::std::env::args(), functions::FUNCTIONS);
}
