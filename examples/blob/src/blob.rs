use azure_functions::bindings::BlobTrigger;
use azure_functions::func;

#[func]
#[binding(name = "trigger", path = "test/{name}")]
pub fn print_blob(trigger: &BlobTrigger) {
    info!("Blob (as string): {:?}", trigger.blob.as_str());
}
