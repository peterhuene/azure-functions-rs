extern crate azure_functions;
#[macro_use]
extern crate log;

mod timer;

azure_functions::register!{
    timer::timer,
}
