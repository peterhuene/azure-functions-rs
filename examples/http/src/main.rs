mod functions;
#[macro_use]
extern crate serde;

pub fn main() {
    azure_functions::worker_main(std::env::args(), functions::FUNCTIONS);
}
