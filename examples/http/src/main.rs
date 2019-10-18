mod functions;

pub fn main() {
    azure_functions::worker_main(std::env::args(), functions::EXPORTS);
}
