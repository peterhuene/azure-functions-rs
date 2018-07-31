use azure_functions::bindings::{Blob, HttpRequest};
use azure_functions::func;

#[func]
#[binding(
    name = "_req",
    auth_level = "anonymous",
    web_hook_type = "generic"
)]
#[binding(name = "blob", path = "copy/{filename}")]
#[binding(name = "$return", path = "copy/{filename}.copy")]
pub fn copy_blob(_req: &HttpRequest, blob: &Blob) -> Blob {
    let contents = blob.contents();

    info!("Blob contents: {:?}", contents.as_str());

    contents.into()
}
