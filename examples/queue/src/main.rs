#![feature(proc_macro_hygiene)]

extern crate azure_functions;
#[macro_use]
extern crate log;

mod queue;
mod queue_with_output;

pub fn main() {
    azure_functions::worker_main(
        ::std::env::args(),
        azure_functions::export!{
            queue::queue,
            queue_with_output::queue_with_output
        },
    );
}
