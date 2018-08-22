extern crate azure_functions;
#[macro_use]
extern crate log;

mod blob_watcher;
mod copy_blob;
mod create_blob;
mod print_blob;

azure_functions::main!{
    blob_watcher::blob_watcher,
    copy_blob::copy_blob,
    create_blob::create_blob,
    print_blob::print_blob,
}
