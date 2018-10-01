extern crate azure_functions;
#[macro_use]
extern crate log;

mod queue;
mod queue_with_output;

azure_functions::register!{
    queue::queue,
    queue_with_output::queue_with_output
}
