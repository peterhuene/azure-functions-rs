#![cfg_attr(feature = "unstable", feature(async_await))]

mod functions;

fn main() {
    azure_functions::worker_main(std::env::args(), functions::EXPORTS);
}
