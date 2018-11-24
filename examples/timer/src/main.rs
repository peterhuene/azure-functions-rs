#![feature(proc_macro_hygiene)]

extern crate azure_functions;
#[macro_use]
extern crate log;

mod timer;

pub fn main() {
    azure_functions::worker_main(
        ::std::env::args(),
        azure_functions::export! {
            timer::timer,
        },
    );
}
