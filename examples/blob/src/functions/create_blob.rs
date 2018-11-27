use azure_functions::bindings::{Blob, HttpRequest, HttpResponse};
use azure_functions::func;
use azure_functions::http::Status;

#[func]
#[binding(
    name = "req",
    auth_level = "anonymous",
    route = "create/blob/{container}/{name}"
)]
#[binding(name = "output1", path = "{container}/{name}")]
pub fn create_blob(req: &HttpRequest) -> (HttpResponse, Blob) {
    (
        HttpResponse::build()
            .status(Status::Created)
            .body("blob has been created.")
            .into(),
        req.body().as_bytes().into(),
    )
}
