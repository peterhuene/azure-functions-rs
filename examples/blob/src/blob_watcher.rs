use azure_functions::bindings::BlobTrigger;
use azure_functions::func;

#[func]
#[binding(name = "trigger", path = "watching/{name}")]
pub fn blob_watcher(trigger: &BlobTrigger) {
    info!(
        "A blob was created at '{}' with contents: {:?}.",
        trigger.path, trigger.blob
    );
}
