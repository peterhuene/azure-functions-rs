use azure_functions::{bindings::BlobTrigger, func};

#[func]
pub fn blob_watcher(#[binding(path = "watching/{name}")] trigger: BlobTrigger) {
    log::info!(
        "A blob was created at '{}' with contents: {:?}.",
        trigger.path,
        trigger.blob
    );
}
