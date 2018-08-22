extern crate azure_functions;
extern crate serde_json;

mod create_row;
mod read_row;

azure_functions::main!{
    create_row::create_row,
    read_row::read_row,
}
