mod blob_watcher;
mod copy_blob;
mod create_blob;
mod print_blob;

pub const FUNCTIONS: &[&azure_functions::codegen::Function] = azure_functions::export! {
    blob_watcher::blob_watcher,
    copy_blob::copy_blob,
    create_blob::create_blob,
    print_blob::print_blob
};
