use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "output1", path = "{container}/{name}.copy")]
pub fn copy_blob(
    #[binding(route = "copy/blob/{container}/{name}")] _req: HttpRequest,
    #[binding(path = "{container}/{name}")] blob: Blob,
) -> (HttpResponse, Blob) {
    ("blob has been copied.".into(), blob)
}
