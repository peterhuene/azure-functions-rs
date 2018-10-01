extern crate azure_functions;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod greet;
mod greet_with_json;

azure_functions::register!{
    greet::greet,
    greet_with_json::greet_with_json
}
