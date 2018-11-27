use azure_functions::bindings::{Blob, HttpRequest, HttpResponse};
use azure_functions::func;

#[func]
#[binding(
    name = "_req",
    auth_level = "anonymous",
    route = "copy/blob/{container}/{name}"
)]
#[binding(name = "blob", path = "{container}/{name}")]
#[binding(name = "output1", path = "{container}/{name}.copy")]
pub fn copy_blob(_req: &HttpRequest, blob: &Blob) -> (HttpResponse, Blob) {
    ("blob has been copied.".into(), blob.clone())
}
