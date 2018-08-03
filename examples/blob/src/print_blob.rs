use azure_functions::bindings::{Blob, HttpRequest, HttpResponse};
use azure_functions::func;

#[func]
#[binding(
    name = "_req",
    auth_level = "anonymous",
    route = "print/blob/{container}/{path}"
)]
#[binding(name = "blob", path = "{container}/{path}")]
pub fn print_blob(_req: &HttpRequest, blob: &Blob) -> HttpResponse {
    blob.as_bytes().into()
}
