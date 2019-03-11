use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "_req", route = "print/blob/{container}/{path}")]
#[binding(name = "blob", path = "{container}/{path}")]
pub fn print_blob(_req: HttpRequest, blob: Blob) -> HttpResponse {
    let response: String = blob.into();
    response.into()
}
