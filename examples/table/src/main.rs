#![feature(proc_macro_hygiene)]

extern crate azure_functions;
extern crate serde_json;

mod create_row;
mod read_row;

pub fn main() {
    azure_functions::worker_main(
        ::std::env::args(),
        azure_functions::export!{
            create_row::create_row,
            read_row::read_row,
        },
    );
}
