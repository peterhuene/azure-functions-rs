mod functions;
mod serialization;

fn main() {
    azure_functions::worker_main(std::env::args(), functions::EXPORTS);
}
