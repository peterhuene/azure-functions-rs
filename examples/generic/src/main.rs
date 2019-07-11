mod functions;

fn main() {
    azure_functions::worker_main_with_extensions(
        std::env::args(),
        functions::EXPORTS,
        &[("Microsoft.Azure.WebJobs.Extensions.CosmosDB", "3.0.3")],
    );
}
