use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn print_blob(
    #[binding(route = "print/blob/{container}/{path}")] _req: HttpRequest,
    #[binding(path = "{container}/{path}")] blob: Blob,
) -> HttpResponse {
    blob.into()
}
